use std::{path::Path, fs};
use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;

use self::flight_data::{FlightData, trace_manager::FlightTrace};

pub mod flight_data;

pub struct FlightManager{
    db_conn: Connection,
}

#[derive(Debug,Clone)]
pub enum Error {
    SqlErr,
    FileErr,
}

impl FlightManager {
    pub fn new() -> Result<Self,Error>
    {
        let flight_manager: FlightManager = FlightManager { 
            // db_conn: Connection::open_in_memory().unwrap(), //Open path
            db_conn: match Connection::open("./flight_database.db")
            {
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

        return Ok(flight_manager);
    }
    
    pub fn load(&self,path: &Path) -> Vec<Result<FlightData,Error>>
    {
        let flights: &mut Vec<Result<FlightData,Error>> = &mut Vec::new();
        Self::search_igc(&path, flights,&|igc_file| FlightData::from_igc(igc_file));
        
        return flights.to_vec();
    }

    pub fn store(&self, flights: Vec<Result<FlightData,Error>>)
    {
        for flight_res in flights
        {
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

    pub fn history(&self, year: Option<u32>, month: Option<u32>) -> Vec<FlightData>
    {
        let mut fligths: Vec<FlightData> = Vec::new();
        let mut stmt: rusqlite::Statement<'_>;

        let mut sql = "SELECT id, date, duration, distance, tags FROM flights ORDER BY date DESC".to_string();

        if year.is_some()
        {
            if month.is_some()
            {
                sql.push_str(format!(" WHERE date>={}-{}-01",year.unwrap(),month.unwrap()).as_str());
            }else {
                sql.push_str(format!(" WHERE date>={}-01-01",year.unwrap()).as_str());
            }
        }
        

        match self.db_conn.prepare(&sql) {
            Ok(s) => stmt = s,
            Err(e) => {
                println!("{:?}",e);
                return fligths
            },
        }

        let rows = stmt.query_map([], |row| {
            Ok(FlightData{
                id: row.get(0).unwrap_or(None),
                date: NaiveDate::parse_from_str(&row.get::<usize,String>(1).unwrap_or("0-01-01".to_string())[..], "%Y-%m-%d").unwrap(),
                duration: row.get(2).unwrap_or(0),
                distance: row.get(3).unwrap_or(0),
                tags: row.get(4).unwrap_or(None),
                hash: "".to_string(),
                takeoff: None,
                landing: None,
                points: None,
                trace: None,
                wing: "".to_string(),
            })
        }).unwrap();

        // for f in rows
        // {
        //     println!("{}",&f.unwrap().date);
        // }

        for flight in rows
        {
            match flight {
                Ok(f) => fligths.push(f),
                Err(_) => (),
            };
        }

        return fligths;
    }

    fn search_igc<F,T>(path: &Path, output: &mut Vec<T>, f: &F) where
        F: Fn(String) -> T
    {
        if match fs::metadata(path) {
            Ok(md) => md.is_dir(),
            Err(_) => false,
        }
        {
            let dir: fs::ReadDir = match fs::read_dir(path){
                Ok(s) => s,
                Err(_) => return,
            };
            for subdir in dir
            {
                match subdir {
                    Ok(s) => Self::search_igc(&s.path(),output,f),
                    Err(_) => return,
                }
            }
        }else if match fs::metadata(path) {
            Ok(md) => md.is_file(),
            Err(_) => false,
        } 
        {
            if path.extension().map(|s| s == "igc").unwrap_or(false)
            {
                match path.to_str() {
                    Some(s) => output.push(f(s.to_string())),
                    None => return,
                }
            }
        }
    }

}