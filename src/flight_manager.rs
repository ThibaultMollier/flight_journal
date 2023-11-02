use rusqlite::Connection;
use std::{fs, path::Path};

use self::flight_data::FlightData;

mod flight_data;

pub struct FlightManager{
    db_conn: Connection,
}

pub struct  Flight{
    pub id: u32,
    pub data: FlightData,
}

/*
TODO :
- Manage error
*/
impl FlightManager {
    pub fn new() -> Self
    {
        let flight_manager: FlightManager = FlightManager { 
            db_conn: Connection::open_in_memory().unwrap(), //Open path
        };

        //TODO Create table only if db doesn't existe
        flight_manager.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS flights (
                id    INTEGER PRIMARY KEY,
                hash  BLOB,
                date  DATE NOT NULL,
                data  BLOB,
                UNIQUE(hash)
            )",
            (), // empty list of parameters.
        ).unwrap();

        return flight_manager;
    }

    pub fn store(&self, path: &String)
    {
        let hash: &str = Path::new(path).file_name().unwrap().to_str().unwrap();
        let igc: String = fs::read_to_string(path).unwrap();

        let flight: FlightData = FlightData::load(&igc);

        let _ = self.db_conn.execute(
            "INSERT OR IGNORE INTO flights (hash, date, data) VALUES (?1, ?2, ?3)",
            (hash,flight.date, "".to_string()),
        ).unwrap();
    }

    /*
    TODO :
    - Maybe only one year ?
    */
    pub fn history(&self) -> Vec<Flight>
    {
        let mut stmt: rusqlite::Statement<'_> = self.db_conn.prepare("SELECT id, date FROM flights ORDER BY date").unwrap();

        let rows = stmt.query_map([], |row| {
            Ok(Flight{
                id: row.get(0).unwrap(),
                data: FlightData{
                    date: row.get(1).unwrap(),
                },
            })
        }).unwrap();

        let mut fligths: Vec<Flight> = Vec::new();

        for flight in rows
        {
            fligths.push(flight.unwrap());
        }

        return fligths;
    }
}