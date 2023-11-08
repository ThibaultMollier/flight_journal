use chrono::NaiveDate;
use rusqlite::Connection;
use std::{fs, path::Path};

use self::flight_data::{trace_manager::FlightTrace, FlightData, Wing, Site};

pub mod flight_data;

pub struct FlightManager {
    db_conn: Connection,
}

#[derive(Debug, Clone)]
pub enum Error {
    SqlErr,
    FileErr,
}

impl FlightManager {
    pub fn new() -> Result<Self, Error> {
        let flight_manager: FlightManager = FlightManager {
            // db_conn: Connection::open_in_memory().unwrap(), //Open path
            db_conn: match Connection::open("./flight_database.db") {
                Ok(conn) => conn,
                Err(_) => return Err(Error::SqlErr),
            },
        };

        match flight_manager.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS flights (
                id          INTEGER PRIMARY KEY,
                hash        BLOB,
                date        DATE NOT NULL,
                duration    INTEGER,
                distance    INTEGER,
                takeoff     TEXT,
                landing     TEXT,
                tags        TEXT,
                wing        TEXT,
                points      BLOB,
                igc         BLOB,
                UNIQUE(hash)
            )",
            (), // empty list of parameters.
        ) {
            Ok(_) => (),
            Err(_) => return Err(Error::SqlErr),
        }

        match flight_manager.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS wings (
                id          INTEGER,
                name        TEXT,
                info        BLOB,
                UNIQUE(id)
            )",
            (), // empty list of parameters.
        ) {
            Ok(_) => (),
            Err(_) => return Err(Error::SqlErr),
        }

        match flight_manager.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS sites (
                id          INTEGER PRIMARY KEY,
                name        TEXT,
                lat         BLOB,
                long        BLOB,
                info        TEXT,
                UNIQUE(id)
            )",
            (), // empty list of parameters.
        ) {
            Ok(_) => (),
            Err(_) => return Err(Error::SqlErr),
        }

        Ok(flight_manager)
    }

    pub fn load_traces(&self, path: &Path) -> Vec<Result<FlightData, Error>> {
        let flights: &mut Vec<Result<FlightData, Error>> = &mut Vec::new();
        Self::search_igc(path, flights, &FlightData::from_igc);

        flights.to_vec()
    }

    pub fn store_flights(&self, flights: Vec<Result<FlightData, Error>>) {
        for flight_res in flights {
            let points = "";

            match flight_res {
                Ok(flight) =>
                    self.db_conn.execute(
                    "INSERT OR IGNORE INTO flights (hash, date, duration, distance, takeoff, landing, tags, wing, points, igc)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                        (
                            flight.hash,
                            flight.date.format("%Y-%m-%d").to_string(),
                            flight.duration,
                            flight.distance,
                            flight.takeoff,
                            flight.landing,
                            flight.tags,
                            flight.wing,
                            points,
                            flight.trace.unwrap_or(FlightTrace::new("None".to_string())).raw_igc,
                        ),
                    ).unwrap_or(0),
                Err(_) => 0,
            };
        }
    }

    pub fn edit_flight(&self, id: u32, tags: String, takeoff: String, landing: String, wing: String)
    {
        // self.db_conn.execute("UPDATE flights SET tags=?1 WHERE id=?2", [-1,0]);
    }

    pub fn flights_history(&self, year: Option<u32>, month: Option<u32>) -> Vec<FlightData> {
        let mut fligths: Vec<FlightData> = Vec::new();
        let mut stmt: rusqlite::Statement<'_>;

        let mut sql = "SELECT id, date, duration, distance, tags FROM flights".to_string();

        if year.is_some() {
            if month.is_some() {
                sql.push_str(
                    format!(
                        " WHERE strftime('%Y',date)=='{}' AND strftime('%m',date)=='{:02}'",
                        year.unwrap(),
                        month.unwrap()
                    )
                    .as_str(),
                );
            } else {
                sql.push_str(format!(" WHERE strftime('%Y',date)=='{}'", year.unwrap()).as_str());
            }
        }

        sql.push_str(" ORDER BY date DESC");

        println!("{}", sql);

        match self.db_conn.prepare(&sql) {
            Ok(s) => stmt = s,
            Err(e) => {
                println!("{:?}", e);
                return fligths;
            }
        }

        let rows = stmt
            .query_map([], |row| {
                Ok(FlightData {
                    id: row.get(0).unwrap_or(None),
                    hash: "".to_string(),
                    date: NaiveDate::parse_from_str(
                        &row.get::<usize, String>(1).unwrap_or("0-01-01".to_string())[..],
                        "%Y-%m-%d",
                    )
                    .unwrap(),
                    duration: row.get(2).unwrap_or(0),
                    distance: row.get(3).unwrap_or(0),
                    tags: row.get(4).unwrap_or(None),
                    takeoff: None,
                    landing: None,
                    points: None,
                    trace: None,
                    wing: "".to_string(),
                })
            })
            .unwrap();

        for flight in rows {
            if let Ok(f) = flight {
                fligths.push(f)
            }
        }

        fligths
    }

    pub fn get_flights_by_tags(&self, tags: String) -> Vec<FlightData> {
        let mut res: Vec<FlightData> = Vec::new();
        let flights: Vec<FlightData> = self.flights_history(None, None);

        for flight in flights {
            let tag = flight.clone().tags.unwrap_or("none".to_string());
            if tag.to_lowercase().contains(&tags.to_lowercase()) {
                res.push(flight)
            }
        }

        res
    }

    pub fn get_flights_by_sites(&self, sites: String) -> Vec<FlightData> {
        let mut res: Vec<FlightData> = Vec::new();
        let flights: Vec<FlightData> = self.flights_history(None, None);

        for flight in flights {
            let site = flight.clone().takeoff.unwrap_or("none".to_string());
            if site.to_lowercase().contains(&sites.to_lowercase()) {
                res.push(flight.clone())
            }

            let site = flight.clone().landing.unwrap_or("none".to_string());
            if site.to_lowercase().contains(&sites.to_lowercase()) {
                res.push(flight)
            }
        }

        res
    }

    pub fn get_flights_by_id(&self, id: u32) -> FlightData {
        let mut stmt: rusqlite::Statement<'_> = self
            .db_conn
            .prepare("SELECT * FROM flights WHERE id = ?1")
            .unwrap();

        let flight = stmt
            .query_row([id.to_string().as_str()], |row| {
                Ok(FlightData {
                    id: row.get(0).unwrap_or(None),
                    hash: row.get(1).unwrap_or("".to_string()),
                    date: NaiveDate::parse_from_str(
                        &row.get::<usize, String>(2).unwrap_or("0-01-01".to_string())[..],
                        "%Y-%m-%d",
                    )
                    .unwrap(),
                    duration: row.get(3).unwrap_or(0),
                    distance: row.get(4).unwrap_or(0),
                    takeoff: row.get(5).unwrap_or(None),
                    landing: row.get(6).unwrap_or(None),
                    tags: row.get(7).unwrap_or(None),
                    wing: row.get(8).unwrap_or("".to_string()),
                    points: None,
                    trace: FlightData::from_igc(Path::new(
                        &row.get(10).unwrap_or("".to_string()).to_string(),
                    ))
                    .unwrap()
                    .trace,
                })
            })
            .unwrap();

        flight
    }

    pub fn delete_flight(&self, id: u32) {
        let _ = self
            .db_conn
            .execute("DELETE FROM flights WHERE id = ?1", [id])
            .unwrap_or(0);
    }

    pub fn store_wing(&self, wing: Wing) {
        self.db_conn.execute(
        "INSERT OR IGNORE INTO wings (id, name, info)
            VALUES (?1, ?2, ?3)",
            (
                wing.id,
                wing.name,
                wing.info,
            ),
        ).unwrap_or(0);
    }

    pub fn get_wings(&self) -> Vec<Wing>
    {
        let mut wings: Vec<Wing> = Vec::new();
        let mut stmt: rusqlite::Statement<'_>;

        match self.db_conn.prepare("SELECT * FROM wings") {
            Ok(s) => stmt = s,
            Err(e) => {
                println!("{:?}", e);
                return wings;
            }
        }

        let rows = stmt
            .query_map([], |row| {
                Ok(Wing{
                    id: row.get(0).unwrap_or(255),
                    name: row.get(1).unwrap_or("None".to_string()),
                    info: row.get(2).unwrap_or("".to_string()),
                })
            })
            .unwrap();

        for wing in rows {
            if let Ok(f) = wing {
                wings.push(f)
            }
        }

        wings
    }

    pub fn delete_wing(&self, id: i32) 
    {
        let _ = self
            .db_conn
            .execute("DELETE FROM wings WHERE id = ?1", [id])
            .unwrap_or(0);
    }

    pub fn set_default_wing(&self, id:i32)
    {
        match self.db_conn.execute("UPDATE wings SET id=?1 WHERE id=?2", [-1,0]) {
            Ok(_) => (),
            Err(_) => return,
        }

        match self.db_conn.execute("UPDATE wings SET id=?1 WHERE id=?2", [0,id]) {
            Ok(_) => (),
            Err(_) => return,
        }      

        match self.db_conn.execute("UPDATE wings SET id=?1 WHERE id=?2", [id,-1]) {
            Ok(_) => (),
            Err(_) => return,
        }  
    }

    pub fn store_site(&self, site: Site) {
        self.db_conn.execute(
        "INSERT OR IGNORE INTO site (name, lat, long, info)
            VALUES (?1, ?2, ?3, ?4)",
            (
                site.name,
                site.lat,
                site.long,
                site.info,
            ),
        ).unwrap_or(0);
    }

    pub fn get_sites(&self) -> Vec<Site>
    {
        let mut sites: Vec<Site> = Vec::new();
        let mut stmt: rusqlite::Statement<'_>;

        match self.db_conn.prepare("SELECT * FROM sites") {
            Ok(s) => stmt = s,
            Err(e) => {
                println!("{:?}", e);
                return sites;
            }
        }

        let rows = stmt
            .query_map([], |row| {
                Ok(Site{
                    id: row.get(0).unwrap_or(255),
                    name: row.get(1).unwrap_or("None".to_string()),
                    lat: row.get(2).unwrap_or(0.0),
                    long: row.get(3).unwrap_or(0.0),
                    info: row.get(4).unwrap_or("".to_string()),
                })
            })
            .unwrap();

        for site in rows {
            if let Ok(f) = site {
                sites.push(f)
            }
        }

        sites
    }

    pub fn delete_site(&self, id: i32) 
    {
        let _ = self
            .db_conn
            .execute("DELETE FROM sites WHERE id = ?1", [id])
            .unwrap_or(0);
    }

    fn search_igc<F, T>(path: &Path, output: &mut Vec<T>, f: &F)
    where
        F: Fn(String) -> T,
    {
        if match fs::metadata(path) {
            Ok(md) => md.is_dir(),
            Err(_) => false,
        } {
            let dir: fs::ReadDir = match fs::read_dir(path) {
                Ok(s) => s,
                Err(_) => return,
            };
            for subdir in dir {
                match subdir {
                    Ok(s) => Self::search_igc(&s.path(), output, f),
                    Err(_) => return,
                }
            }
        } else if match fs::metadata(path) {
            Ok(md) => md.is_file(),
            Err(_) => false,
        } && path.extension().map(|s| s == "igc").unwrap_or(false)
        {
            match path.to_str() {
                Some(s) => output.push(f(s.to_string())),
                None => return,
            }
        }
    }
}
