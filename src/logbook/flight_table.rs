use chrono::NaiveDate;
use rusqlite::Connection;
use anyhow::{Result, bail};

use super::{FlightPoint, DATABASE_PATH, IDListe};

#[derive(Clone,Debug)]
pub struct FlightTable{
    pub flight_id   :u32,
    pub wing_id     :u32,
    pub takeoff_id  :u32,
    pub landing_id  :u32,
    pub hash        :String,
    pub date        :NaiveDate,
    pub duration    :u32,
    pub distance    :u32,
    pub points      :Option<Vec<FlightPoint>>,
    pub raw_igc     :Option<String>
}

impl FlightTable {
    pub fn create() -> Result<()>{
        let db_conn = Connection::open(DATABASE_PATH)?;

        db_conn.execute("PRAGMA foreign_keys = ON;",())?;

        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS flights (
                flight_id   INTEGER PRIMARY KEY,
                wing_id     INTEGER REFERENCES wings(wing_id),
                takeoff_id  INTEGER REFERENCES sites(site_id),
                landing_id  INTEGER REFERENCES sites(site_id),
                hash        BLOB UNIQUE,
                date        DATE NOT NULL,
                duration    INTEGER,
                distance    INTEGER,
                points      BLOB,
                igc         BLOB
            );",
            (), // empty list of parameters.
        )?;
        db_conn.close().unwrap_or_default();
        Ok(())
    }

    pub fn store(flight: FlightTable) -> Result<()>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(
            "INSERT INTO flights (hash, date, duration, distance, takeoff_id, landing_id, wing_id, points, igc)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                (
                    flight.hash,
                    flight.date.format("%Y-%m-%d").to_string(),
                    flight.duration,
                    flight.distance,
                    flight.takeoff_id,
                    flight.landing_id,
                    flight.wing_id,
                    "",
                    flight.raw_igc,
                ),
            )?;

        db_conn.close().unwrap_or_default();
        Ok(())
    }

    pub fn get(id: u32) -> Result<FlightTable>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut stmt: rusqlite::Statement<'_> = db_conn.prepare("SELECT * FROM flights WHERE flight_id=?1")?;

        let flight = stmt
            .query_row([id], |row| {
                Ok(FlightTable {
                    flight_id: row.get(0)?,
                    wing_id: row.get(1)?,
                    takeoff_id: row.get(2)?,
                    landing_id: row.get(3)?,
                    hash: row.get(4)?,
                    date: NaiveDate::parse_from_str(&row.get::<usize, String>(5)?, "%Y-%m-%d").unwrap_or_default(),
                    duration: row.get(6)?,
                    distance: row.get(7)?,
                    points: None,
                    raw_igc: row.get(9)?,
                })
            })?;

        Ok(flight)
    }

    pub fn delete(condition: String) -> Result<()>
    {
        let mut sql = "DELETE FROM flights WHERE ".to_string();
        sql.push_str(&condition);
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(&sql,())?;

        db_conn.close().unwrap_or_default();

        Ok(())
    }

    pub fn update(set: String, condition: String) -> Result<()>
    {
        let mut sql = "UPDATE flights SET ".to_string();
        sql.push_str(&set);
        sql.push_str(" WHERE ");
        sql.push_str(&condition);
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(&sql,())?;

        db_conn.close().unwrap_or_default();

        Ok(())
    }

    pub fn select(condition: String) -> Result<Vec<FlightTable>>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut fligths: Vec<FlightTable> = Vec::new();
        let mut sql = "SELECT flight_id, takeoff_id, landing_id, date, duration, distance FROM flights WHERE ".to_string();
        sql.push_str(&condition);

        let mut stmt = db_conn.prepare(&sql)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(FlightTable {
                    flight_id: row.get(0)?,
                    wing_id: 0,
                    takeoff_id: row.get(1)?,
                    landing_id: row.get(2)?,
                    hash: "".to_string(),
                    date: NaiveDate::parse_from_str(&row.get::<usize, String>(3)?, "%Y-%m-%d").unwrap_or_default(),
                    duration: row.get(4)?,
                    distance: row.get(5)?,
                    points: None,
                    raw_igc: None,
                })
            })?;

        for flight in rows {
            if let Ok(f) = flight {
                fligths.push(f)
            }
        }

        Ok(fligths)
    }

    pub fn select_all() -> Result<Vec<FlightTable>>
    {
        FlightTable::select("1".to_string())
    }

    pub fn get_by_tag(tag_ids: IDListe) -> Result<Vec<FlightTable>>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut fligths: Vec<FlightTable> = Vec::new();
        let sql = format!("SELECT flight_id, date, duration, distance, takeoff_id, landing_id FROM flights INNER JOIN tag_asso WHERE flight_id=asso_flight_id AND tag_id IN {};",tag_ids.to_string());
        let mut stmt = db_conn.prepare(&sql)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(FlightTable {
                    flight_id: row.get(0)?,
                    wing_id: 0,
                    takeoff_id: row.get(1)?,
                    landing_id: row.get(2)?,
                    hash: "".to_string(),
                    date: NaiveDate::parse_from_str(&row.get::<usize, String>(3)?, "%Y-%m-%d").unwrap_or_default(),
                    duration: row.get(4)?,
                    distance: row.get(5)?,
                    points: None,
                    raw_igc: None,
                })
            })?;

        for flight in rows {
            if let Ok(f) = flight {
                fligths.push(f)
            }
        }

        Ok(fligths)
    }
}