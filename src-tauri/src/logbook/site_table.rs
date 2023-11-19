use geoutils::Location;
use rusqlite::Connection;
use anyhow::{Result, bail};

use super::{DATABASE_PATH, IDListe, FlightPoint};

const DISTANCE_DETECTION: f64 = 200.0;

#[derive(Clone)]
pub struct SiteTable
{
    pub site_id: u32,
    pub name: String,
    pub lat: f32,
    pub long: f32,
    pub alt: u32,
    pub info: String,
}

impl SiteTable
{
    pub fn create() -> Result<()>{
        let db_conn = Connection::open(DATABASE_PATH)?;

        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS sites (
                site_id     INTEGER PRIMARY KEY,
                name        TEXT,
                lat         FLOAT,
                long        FLOAT,
                alt         INTEGER,
                info        TEXT
            );",
            (), // empty list of parameters.
        )?;
        db_conn.close().unwrap_or_default();
        Ok(())
    }

    pub fn store(site: SiteTable) -> Result<()>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(
            "INSERT INTO sites (name, lat, long, alt, info)
                VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    site.name,
                    site.lat,
                    site.long,
                    site.alt,
                    site.info,
                ),
            )?;

        db_conn.close().unwrap_or_default();
        Ok(())
    }

    pub fn get(id: u32) -> Result<SiteTable>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut stmt: rusqlite::Statement<'_> = db_conn.prepare("SELECT * FROM sites WHERE site_id=?1")?;

        let site = stmt
            .query_row([id], |row| {
                Ok(SiteTable {
                    site_id: row.get(0)?,
                    name: row.get(1)?,
                    lat: row.get(2)?,
                    long: row.get(3)?,
                    alt: row.get(4)?,
                    info: row.get(5)?,
                })
            })?;

        Ok(site)
    }

    pub fn delete(condition: String) -> Result<()>
    {
        let mut sql = "DELETE FROM sites WHERE ".to_string();
        sql.push_str(&condition);
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(&sql,())?;

        db_conn.close().unwrap_or_default();

        Ok(())
    }

    pub fn update(set: String, condition: String) -> Result<()>
    {
        let mut sql = "UPDATE sites SET ".to_string();
        sql.push_str(&set);
        sql.push_str(" WHERE ");
        sql.push_str(&condition);
        let db_conn = Connection::open(DATABASE_PATH)?;
        db_conn.execute(&sql,())?;

        db_conn.close().unwrap_or_default();

        Ok(())
    }

    pub fn select(condition: String) -> Result<Vec<SiteTable>>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut sites: Vec<SiteTable> = Vec::new();
        let mut sql = "SELECT * FROM sites WHERE ".to_string();
        sql.push_str(&condition);

        let mut stmt = db_conn.prepare(&sql)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SiteTable {
                    site_id: row.get(0)?,
                    name: row.get(1)?,
                    lat: row.get(2)?,
                    long: row.get(3)?,
                    alt: row.get(4)?,
                    info: row.get(5)?,
                })
            })?;

        for site in rows {
            if let Ok(s) = site {
                sites.push(s)
            }
        }

        Ok(sites)
    }

    pub fn select_all() -> Result<Vec<SiteTable>>
    {
        SiteTable::select("1".to_string())
    }

    pub fn search(search: String) -> Result<IDListe>
    {
        let sites = SiteTable::select_all()?;
        let mut site_ids:IDListe = IDListe { list: Vec::new() };

        for site in sites
        {
            if site.name.to_lowercase().contains(&search.to_lowercase())
            {
                site_ids.list.push(site.site_id);
            }
        }

        Ok(site_ids)
    }

    pub fn site_detection(takeoff: FlightPoint, landing: FlightPoint) -> Result<(Option<SiteTable>, Option<SiteTable>)>
    {
        let sites = Self::select_all()?;
        let mut res_takeoff: Option<SiteTable> = None;
        let mut res_landing: Option<SiteTable> = None;

        let takeoff_loc = Location::new(takeoff.lat,takeoff.long);
        let landing_loc = Location::new(landing.lat,landing.long);

        for site in sites
        {
            let site_loc = Location::new(site.lat,site.long);
            let d = takeoff_loc.haversine_distance_to(&site_loc).meters();

            if d < DISTANCE_DETECTION
            {
                res_takeoff = Some(site.clone());
            }

            let d = landing_loc.haversine_distance_to(&site_loc).meters();
            if d < DISTANCE_DETECTION
            {
                res_landing = Some(site.clone());
            }

            if res_landing.is_some() && res_takeoff.is_some()
            {
                break;
            }
        }

        return Ok((res_takeoff,res_landing));
    }

    pub fn last_site_id() -> Result<u32>
    {
        let db_conn = Connection::open(DATABASE_PATH)?;
        let mut stmt = db_conn.prepare("SELECT site_id FROM sites ORDER BY site_id DESC LIMIT 1;")?;

        let id = stmt.query_row([], | row | Ok(row.get(0).unwrap_or(0)))?;

        Ok(id)
    }
}