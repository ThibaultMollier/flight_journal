use chrono::NaiveDate;
use rusqlite::Connection;
use std::{fs, path::Path};
use serde_json::Value;
use self::flight_data::{trace_manager::FlightTrace, FlightData, FlightCompute, Wing, Site};

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
    // Open sql connection and create tables if not exist
    pub fn new() -> Result<Self, Error> {
        let flight_manager: FlightManager = FlightManager {
            // db_conn: Connection::open_in_memory().unwrap(), //Open path
            db_conn: match Connection::open("./flight_database.db") {
                Ok(conn) => conn,
                Err(_) => return Err(Error::SqlErr),
            },
        };

        flight_manager.db_conn.execute("PRAGMA foreign_keys = ON;",()).unwrap();

        match flight_manager.db_conn.execute(
            "CREATE TABLE wings (
                wing_id     INTEGER PRIMARY KEY,
                name        TEXT,
                info        TEXT
            );",
            (), // empty list of parameters.
        ) {
            Ok(_) => {flight_manager.db_conn.execute("INSERT INTO wings (wing_id,name,info) VALUES (0,'default','')",()).unwrap_or(0);},
            Err(e) => {
                dbg!(e);},
        }

        let sites = match flight_manager.db_conn.execute(
            "CREATE TABLE sites (
                site_id     INTEGER PRIMARY KEY,
                name        TEXT,
                lat         FLOAT,
                long        FLOAT,
                alt         INTEGER,
                info        TEXT DEFAULT 'default'
            );",
            (), // empty list of parameters.
        ) {
            Ok(_) => Self::get_ffvl_site(),
            Err(e) => {
                dbg!(e);
                Vec::new()},
        };

        for site in sites {
            flight_manager.store_site(site).unwrap();
        }

        match flight_manager.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS flights (
                flight_id    INTEGER PRIMARY KEY,
                wing_id     INTEGER,
                takeoff_id  INTEGER,
                landing_id  INTEGER,
                hash        BLOB,
                date        DATE NOT NULL,
                duration    INTEGER,
                distance    INTEGER,
                points      BLOB,
                igc         BLOB,
                UNIQUE(hash),
                FOREIGN KEY(takeoff_id) REFERENCES sites(site_id),
                FOREIGN KEY(landing_id) REFERENCES sites(site_id),
                FOREIGN KEY(wing_id) REFERENCES wings(wing_id)
            );",
            (), // empty list of parameters.
        ) {
            Ok(_) => (),
            Err(e) => {
                dbg!(e);
                return Err(Error::SqlErr)},
        }

        Ok(flight_manager)
    }

    //Open igc file(s) and extract data from it 
    pub fn load_traces(&self, path: &Path) -> Vec<Result<FlightData, Error>> {
        let paths: &mut Vec<String> = &mut Vec::new();
        let flights: &mut Vec<Result<FlightData, Error>> = &mut Vec::new();
        Self::search_igc(path, paths);

        for path in paths 
        {
            flights.push(self.from_igc(path));
        }

        flights.to_vec()
    }

    //Store flight(s) into database
    pub fn store_flights(&self, flights: Vec<Result<FlightData, Error>>) {
        for flight_res in flights {
            let points = "";

            match flight_res {
                Ok(flight) =>
                    match self.db_conn.execute(
                    "INSERT INTO flights (hash, date, duration, distance, takeoff_id, landing_id, wing_id, points, igc)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        (
                            flight.hash,
                            flight.date.format("%Y-%m-%d").to_string(),
                            flight.duration,
                            flight.distance,
                            flight.takeoff,
                            flight.landing,
                            flight.wing,
                            points,
                            flight.trace.unwrap().raw_igc,
                        ),
                    )
                    {
                        Ok(_) => 0,
                        Err(e) => {
                            dbg!(e);
                            0
                        }
                    },
                Err(e) => {
                    dbg!(e);
                    0
                },
            };
        }
    }

    pub fn edit_flight(&self, id: u32, takeoff: Option<u32>, landing: Option<u32>, wing: Option<u32>) -> Result<(),Error>
    {
        let mut sql = "UPDATE flights SET ".to_string();

        if takeoff.is_some()
        {
            sql.push_str(format!("takeoff_id={} ",takeoff.unwrap()).as_str());
        }
        if landing.is_some()
        {
            sql.push_str(format!("landig_id={} ",landing.unwrap()).as_str());
        }
        if wing.is_some()
        {
            sql.push_str(format!("wing_id={} ",wing.unwrap()).as_str());
        }

        sql.push_str("WHERE flight_id=?1");

        match self.db_conn.execute(&sql, [id]) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::SqlErr),
        }
    }


    pub fn flights_history(&self, year: Option<u32>, month: Option<u32>) -> Vec<FlightData> {
        let mut fligths: Vec<FlightData> = Vec::new();
        let mut stmt: rusqlite::Statement<'_>;

        let mut sql = "SELECT flight_id, date, duration, distance, takeoff_id, landing_id FROM flights".to_string();

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
                    takeoff: row.get(4).unwrap_or(None),
                    landing: row.get(5).unwrap_or(None),
                    points: None,
                    trace: None,
                    wing: 0,
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

    // pub fn get_flights_by_tags(&self, tags: String) -> Vec<FlightData> {
    //     let mut res: Vec<FlightData> = Vec::new();
    //     let flights: Vec<FlightData> = self.flights_history(None, None);

    //     for flight in flights {
    //         let tag = flight.clone().tags.unwrap_or("none".to_string());
    //         if tag.to_lowercase().contains(&tags.to_lowercase()) {
    //             res.push(flight)
    //         }
    //     }

    //     res
    // }

    pub fn get_flights_by_sites(&self, site_search: String) -> Vec<FlightData> {
        let mut fligths: Vec<FlightData> = Vec::new();
        let sites: Vec<Site> = self.get_sites();
        let mut site_ids: Vec<u32> = Vec::new();
        let mut stmt: rusqlite::Statement<'_>;
        
        for site in sites
        {
            if site.name.to_lowercase().contains(&site_search.to_lowercase())
            {
                site_ids.push(site.id);
            }
        }

        let mut str_site_ids = "(".to_string();

        for id in site_ids
        {
            str_site_ids.push_str(format!("{},",id).as_str());
        }

        str_site_ids.pop();//remove last ','
        str_site_ids.push(')');

        let sql = format!("SELECT flight_id, date, duration, distance, takeoff_id, landing_id FROM flights WHERE takeoff_id IN {} OR landing_id IN {};",str_site_ids.clone(),str_site_ids.clone());

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
                    takeoff: row.get(4).unwrap_or(None),
                    landing: row.get(5).unwrap_or(None),
                    points: None,
                    trace: None,
                    wing: 0,
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

    pub fn get_flights_by_id(&self, id: u32) -> FlightData {
        let mut stmt: rusqlite::Statement<'_> = self
            .db_conn
            .prepare("SELECT * FROM flights WHERE flight_id = ?1")
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
                    wing: row.get(7).unwrap_or(0),
                    points: None,
                    trace: None,
                    // Some(FlightTrace::new(
                    //     row.get(10).unwrap_or("".to_string()).to_string(),
                    // )),
                })
            })
            .unwrap();

        flight
    }

    pub fn delete_flight(&self, id: u32) -> Result<(),Error> 
    {
        match self
        .db_conn
        .execute("DELETE FROM flights WHERE flight_id = ?1", [id]) 
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::SqlErr),
        }
    }

    pub fn store_wing(&self, wing: Wing)  -> Result<(), Error>
    {
        match self.db_conn.execute(
            "INSERT OR IGNORE INTO wings (name, info)
                VALUES (?1, ?2)",
                (
                    wing.name,
                    wing.info,
                ),
            )
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::SqlErr),
        }
    }

    pub fn edit_wing(&self, id: u32, name: Option<String>, info: Option<String>) -> Result<(),Error>
    {
        let mut sql = "UPDATE wings SET ".to_string();

        if name.is_some()
        {
            sql.push_str(format!("name={} ",name.unwrap()).as_str());
        }
        if info.is_some()
        {
            sql.push_str(format!("info={} ",info.unwrap()).as_str());
        }

        sql.push_str("WHERE wing_id=?2");
        
        match self.db_conn.execute(&sql, [id]) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::SqlErr),
        }
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
                    id: row.get(0).unwrap_or(0),
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

    pub fn delete_wing(&self, id: i32) -> Result<(), Error>
    {
        match self
            .db_conn
            .execute("DELETE FROM wings WHERE wing_id = ?1", [id])
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::SqlErr),
        }
    }

    pub fn set_default_wing(&self, id:i32) -> Result<(), Error>
    {
        // match self.db_conn.execute("UPDATE wings SET id=?1 WHERE id=?2", [-1,0]) {
        //     Ok(_) => (),
        //     Err(_) => return Err(Error::SqlErr),
        // }

        // match self.db_conn.execute("UPDATE wings SET id=?1 WHERE id=?2", [0,id]) {
        //     Ok(_) => (),
        //     Err(_) => return Err(Error::SqlErr),
        // }      

        // match self.db_conn.execute("UPDATE wings SET id=?1 WHERE id=?2", [id,-1]) {
        //     Ok(_) => (),
        //     Err(_) => return Err(Error::SqlErr),
        // }  

        Ok(())
    }

    pub fn store_site(&self, site: Site) -> Result<(), Error>
    {
        match self.db_conn.execute(
        "INSERT OR IGNORE INTO sites (name, lat, long, alt, info)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                site.name,
                site.lat,
                site.long,
                site.alt,
                site.info,
            ),
        )
        {
            Ok(_) => Ok(()),
            Err(e) => {dbg!(e);Err(Error::SqlErr)},
        }
    }

    pub fn edit_site(&self, id: u32, name: Option<String>, lat: Option<f32>, long: Option<f32>, alt: Option<u32>, info: Option<String>) -> Result<(),Error>
    {
        let mut sql = "UPDATE sites SET ".to_string();

        if name.is_some()
        {
            sql.push_str(format!("name={} ",name.unwrap()).as_str());
        }
        if lat.is_some()
        {
            sql.push_str(format!("lat={} ",lat.unwrap()).as_str());
        }
        if long.is_some()
        {
            sql.push_str(format!("long={} ",long.unwrap()).as_str());
        }
        if alt.is_some()
        {
            sql.push_str(format!("alt={} ",alt.unwrap()).as_str());
        }
        if info.is_some()
        {
            sql.push_str(format!("info={} ",info.unwrap()).as_str());
        }

        sql.push_str("WHERE site_id=?2");
        
        match self.db_conn.execute(&sql, [id]) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::SqlErr),
        }
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
                    alt: row.get(4).unwrap_or(0),
                    info: row.get(5).unwrap_or("".to_string()),
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

    pub fn delete_site(&self, id: i32) -> Result<(), Error>
    {
        match self
            .db_conn
            .execute("DELETE FROM sites WHERE site_id = ?1", [id])
        {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::SqlErr),
        }
    }

    fn search_igc(path: &Path, output: &mut Vec<String>)
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
                    Ok(s) => Self::search_igc(&s.path(), output),
                    Err(_) => return,
                }
            }
        } else if match fs::metadata(path) {
            Ok(md) => md.is_file(),
            Err(_) => false,
        } && path.extension().map(|s| s == "igc").unwrap_or(false)
        {
            match path.to_str() {
                Some(s) => output.push(s.to_string()),
                None => return,
            }
        }
    }

    pub fn get_ffvl_site() -> Vec<Site>
    {
        let mut sites: Vec<Site> = Vec::new();
        let resp = reqwest::blocking::get("https://data.ffvl.fr/api/?base=terrains&mode=json&key=00000000000000000000000000000000");
        
        let resp = match resp {
            Ok(s) => s,
            Err(_) => return sites,
        };

        let v: Value = serde_json::from_str(resp.text().unwrap_or("".to_string()).as_str()).unwrap();
        
        for site in 0..v.as_array().unwrap().len()
        {
            sites.push(
                Site
                {
                    id: site as u32,
                    name: v[site]["toponym"].as_str().unwrap_or("").to_string(),
                    lat: v[site]["latitude"].as_str().unwrap_or("0.0").parse::<f32>().unwrap_or(0.0),
                    long: v[site]["longitude"].as_str().unwrap_or("0.0").parse::<f32>().unwrap_or(0.0),
                    alt: v[site]["altitude"].as_str().unwrap_or("0.0").parse::<u32>().unwrap_or(0),
                    info: v[site]["warnings"].as_str().unwrap_or("").to_string(),
                }
            )
        }
        sites
    }
}
