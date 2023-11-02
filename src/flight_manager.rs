use rusqlite::Connection;
use std::{fs, path::Path};

pub struct FlightManager{
    db_conn: Connection,
}

pub struct  Flight{
    id: u32,
    date: String,
    raw_data: String,
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
            "CREATE TABLE flights IF NOT EXISTS (
                id    INTEGER PRIMARY KEY,
                date  DATE NOT NULL,
                data  BLOB
            )",
            (), // empty list of parameters.
        ).unwrap();

        return flight_manager;
    }

    /*
    TODO : 
    - It must be possible to add multiple tarck (ex pass a directory pass and add all igc file in this diretory)
    
     */
    pub fn store<P: AsRef<Path>>(&self, path: P)
    {
        let igc = fs::read_to_string(path).unwrap();

        let _ = self.db_conn.execute(
            "INSERT INTO flight (date, data) VALUES (?1, ?2)",
            ("08-06-2023", igc),
        ).unwrap();
    }

    /*
    TODO :
    - Return a list (vec of struct ?) of  all element in database 
    - Maybe only one year ?
    */
    pub fn history(&self) -> Vec<Flight>
    {
        let mut stmt = self.db_conn.prepare("SELECT id, date FROM flight").unwrap();

        let rows = stmt.query_map([], |row| {
            Ok(Flight{
                id: row.get(0).unwrap(),
                date: row.get(1).unwrap(),
                raw_data : "".to_string(),
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