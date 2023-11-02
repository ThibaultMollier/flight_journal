use rusqlite::Connection;

pub struct FlightManager{
    db_conn: Connection,
}

impl FlightManager {
    pub fn new(&self) -> Self
    {
        let flight_manager: FlightManager = FlightManager { 
            db_conn: Connection::open_in_memory().unwrap(),
        };

        flight_manager.db_conn.execute(
            "CREATE TABLE flights (
                id    INTEGER PRIMARY KEY,
                date  DATE NOT NULL,
                data  BLOB
            )",
            (), // empty list of parameters.
        ).unwrap();

        return flight_manager;
        
    }
}