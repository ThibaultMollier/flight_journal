use chrono::NaiveDate;
use rusqlite::Connection;
use std::{fs, path::Path};
use serde_json::Value;
use self::flight_data::{trace_manager::FlightTrace, FlightData, FlightCompute, Wing, Site, Tag};
use anyhow::{Result, bail};

pub mod flight_data;

pub struct FlightManager {
    db_conn: Connection,
}

impl FlightManager {
    // Open sql connection and create tables if not exist
    pub fn new() -> Result<Self> {

        let flight_manager: FlightManager = FlightManager {
            // db_conn: Connection::open_in_memory()?,//for test only
            db_conn: Connection::open("./flight_database.db")?,
        };

        flight_manager.db_conn.execute("PRAGMA foreign_keys = ON;",())?;

        match flight_manager.db_conn.execute(
            "CREATE TABLE wings (
                wing_id     INTEGER PRIMARY KEY,
                name        TEXT UNIQUE,
                info        TEXT,
                def         BOOLEAN
            );",
            (), // empty list of parameters.
        ) {
            Ok(_) => {
                // First creation of table, insert default wing
                flight_manager.db_conn.execute("INSERT INTO wings (wing_id,name,info,def) VALUES (0,'default','',1)",())?;
            },
            Err(e) => {
                //Table already exist don't pannic
                dbg!(e.to_string());
            }
        }

        match flight_manager.db_conn.execute(
            "CREATE TABLE sites (
                site_id     INTEGER PRIMARY KEY,
                name        TEXT,
                lat         FLOAT,
                long        FLOAT,
                alt         INTEGER,
                info        TEXT
            );",
            (), // empty list of parameters.
        ) {
            Ok(_) => {
                // First creation of table, fill database with ffvl sites
                // let sites = Self::get_ffvl_site();
                // for site in sites {
                //     flight_manager.store_site(site).unwrap();
                // };
            },
            Err(e) => {
                //Table already exist don't pannic
                dbg!(e.to_string());
            },
        };

        flight_manager.db_conn.execute(
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

        flight_manager.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                tag_id      INTEGER PRIMARY KEY,
                name        TEXT UNIQUE
            );",
            (), // empty list of parameters.
        )?;

        flight_manager.db_conn.execute(
            "CREATE TABLE IF NOT EXISTS tag_asso (
                join_id     INTEGER PRIMARY KEY,
                tag_id      INTEGER REFERENCES tags(tag_id),
                flight_id   INTEGER REFERENCES flights(flight_id) 
            );",
            (), // empty list of parameters.
        )?;

        Ok(flight_manager)
    }

    pub fn close(self) -> Result<()>
    {
        self.db_conn.close().unwrap();
        Ok(())
    }

    //Open igc file(s) and extract data from it 
    pub fn load_traces(&self, path: &Path) -> Vec<FlightData> {
        let paths: &mut Vec<String> = &mut Vec::new();
        let flights = &mut Vec::new();
        Self::search_igc(path, paths);

        for path in paths 
        {
            match self.from_igc(path) {
                Ok(f) => flights.push(f),
                Err(e) => {
                    dbg!(e.to_string());
                },
            }            
        }

        flights.to_vec()
    }

    //Store flight(s) into database
    pub fn store_flights(&self, flights: Vec<FlightData>) -> Result<()> {
        for flight in flights {
            let points = "";

            self.db_conn.execute(
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
            )?;
        }

        Ok(())
    }

    pub fn edit_flight(&self, id: u32, takeoff: Option<u32>, landing: Option<u32>, wing: Option<u32>) -> Result<()>
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

        self.db_conn.execute(&sql, [id])?;

        Ok(())
    }

    pub fn flights_history(&self, year: Option<u32>, month: Option<u32>) -> Result<Vec<FlightData>> {
        let mut fligths: Vec<FlightData> = Vec::new();

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

        let mut stmt = self.db_conn.prepare(&sql)?;

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
            })?;

        for flight in rows {
            if let Ok(f) = flight {
                fligths.push(f)
            }
        }

        Ok(fligths)
    }

    pub fn get_flights_by_tags(&self, tag_search: String) -> Result<Vec<FlightData>> {
        let mut flights: Vec<FlightData> = Vec::new();
        let tags = self.get_tags()?;
        let mut tag_ids = Vec::new();

        for tag in tags
        {
            if tag.name.to_lowercase().contains(&tag_search.to_lowercase())
            {
                tag_ids.push(tag.id);
            }
        }

        let mut str_tags_ids = "(".to_string();

        for id in tag_ids
        {
            str_tags_ids.push_str(format!("{},",id).as_str());
        }

        str_tags_ids.pop();//remove last ','
        str_tags_ids.push(')');

        let sql = format!("SELECT flights.flight_id, date, duration, distance, takeoff_id, landing_id FROM flights INNER JOIN tag_asso WHERE flights.flight_id=tag_asso.flight_id AND tag_id IN {};",str_tags_ids);

        let mut stmt = self.db_conn.prepare(&sql)?;

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
            })?;

        for flight in rows {
            if let Ok(f) = flight {
                flights.push(f)
            }
        }

        Ok(flights)
    }

    pub fn get_flights_by_wing(&self, wing_id: u32) -> Result<Vec<FlightData>> {
        let mut flights: Vec<FlightData> = Vec::new();

        let mut stmt = self.db_conn.prepare("SELECT flight_id, date, duration, distance, takeoff_id, landing_id FROM flights WHERE wing_id=?1;")?;

        let rows = stmt
            .query_map([wing_id], |row| {
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
            })?;

        for flight in rows {
            if let Ok(f) = flight {
                flights.push(f)
            }
        }

        Ok(flights)
    }

    pub fn get_flights_by_sites(&self, site_search: String) -> Result<Vec<FlightData>> {
        let mut fligths: Vec<FlightData> = Vec::new();
        let sites: Vec<Site> = self.get_sites()?;
        let mut site_ids: Vec<u32> = Vec::new();
        
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

        let mut stmt = self.db_conn.prepare(&sql)?;

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
            })?;

        for flight in rows {
            if let Ok(f) = flight {
                fligths.push(f)
            }
        }

        Ok(fligths)
    }

    pub fn get_flights_by_id(&self, id: u32) -> Result<FlightData> {
        let mut stmt: rusqlite::Statement<'_> = self
            .db_conn
            .prepare("SELECT * FROM flights WHERE flight_id = ?1")?;

        let flight = stmt
            .query_row([id.to_string().as_str()], |row| {
                Ok(FlightData {
                    id: row.get(0).unwrap_or(None),
                    hash: row.get(4).unwrap_or("".to_string()),
                    date: NaiveDate::parse_from_str(
                        &row.get::<usize, String>(5).unwrap_or("0-01-01".to_string())[..],
                        "%Y-%m-%d",
                    )
                    .unwrap(),
                    duration: row.get(6)?,
                    distance: row.get(7).unwrap_or(0),
                    takeoff: row.get(2).unwrap_or(None),
                    landing: row.get(3).unwrap_or(None),
                    wing: row.get(1).unwrap_or(0),
                    points: None,
                    trace: Some(FlightTrace::new(
                        row.get(9)?,
                    )),
                })
            })?;

        Ok(flight)
    }

    pub fn delete_flight(&self, id: u32) -> Result<()> 
    {
        self
        .db_conn
        .execute("DELETE FROM flights WHERE flight_id = ?1", [id])?;

        Ok(())
    }

    pub fn store_wing(&self, wing: Wing)  -> Result<()>
    {
        self.db_conn.execute(
        "INSERT OR IGNORE INTO wings (name, info)
            VALUES (?1, ?2)",
            (
                wing.name.clone(),
                wing.info,
            ),
        )?;

        if wing.default.unwrap_or(false)
        {
            self.set_default_wing(None,Some(wing.name))?;
        }

        Ok(())
    }

    pub fn edit_wing(&self, id: u32, name: Option<String>, info: Option<String>) -> Result<()>
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
        
        self.db_conn.execute(&sql, [id])?;

        Ok(())
    }

    pub fn get_wings(&self) -> Result<Vec<Wing>>
    {
        let mut wings: Vec<Wing> = Vec::new();
        let mut stmt = self.db_conn.prepare("SELECT * FROM wings")?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Wing{
                    id: row.get(0).unwrap_or(0),
                    name: row.get(1).unwrap_or("None".to_string()),
                    info: row.get(2).unwrap_or("".to_string()),
                    default: row.get(3).unwrap_or(None),
                })
            })?;

        for wing in rows {
            if let Ok(f) = wing {
                wings.push(f)
            }
        }

        Ok(wings)
    }

    pub fn get_wing(&self, id: Option<u32>, name: Option<String>) -> Result<Wing>
    {
        if let Some(i) = id
        {
            let mut stmt = self.db_conn.prepare("SELECT * FROM wings WHERE wing_id=?1")?;
            let wing = stmt.query_row([i], |row| Ok(Wing{
                id: row.get(0).unwrap_or(0),
                name: row.get(1).unwrap_or("".to_string()),
                info: row.get(2).unwrap_or("".to_string()),
                default: row.get(3).unwrap_or(None),
            }))?;
            return Ok(wing);
        }else if let Some(n) = name {
            let mut stmt = self.db_conn.prepare("SELECT * FROM wings WHERE name=?1")?;
            let wing = stmt.query_row([n], |row| Ok(Wing{
                id: row.get(0).unwrap_or(0),
                name: row.get(1).unwrap_or("".to_string()),
                info: row.get(2).unwrap_or("".to_string()),
                default: row.get(3).unwrap_or(None),
            }))?;
            return Ok(wing);
        }
    
        bail!("Paramters error");
    }

    pub fn delete_wing(&self, id: i32) -> Result<()>
    {
        self
            .db_conn
            .execute("DELETE FROM wings WHERE wing_id = ?1", [id])?;

        Ok(())
    }

    pub fn set_default_wing(&self, id:Option<i32>, name: Option<String>) -> Result<()>
    {
        self.db_conn.execute("UPDATE wings SET def=?1 WHERE def=?2", [0,1])?;

        if let Some(i) = id
        {
            self.db_conn.execute("UPDATE wings SET def=?1 WHERE wing_id=?2", (true,i))?;
        }else if let Some(n) = name{
            self.db_conn.execute("UPDATE wings SET def=?1 WHERE name=?2", (true,n))?;
        }
        

        Ok(())
    }

    pub fn store_site(&self, site: Site) -> Result<()>
    {
        self.db_conn.execute(
        "INSERT OR IGNORE INTO sites (name, lat, long, alt, info)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                site.name,
                site.lat,
                site.long,
                site.alt,
                site.info,
            ),
        )?;

        Ok(())
    }

    pub fn last_site_id(&self) -> Result<u32>
    {
        let mut stmt = self.db_conn.prepare("SELECT site_id FROM sites ORDER BY site_id DESC LIMIT 1;")?;

        let id = stmt.query_row([], | row | Ok(row.get(0).unwrap_or(0)))?;

        Ok(id)
    }

    pub fn edit_site(&self, id: u32, name: Option<String>, lat: Option<f32>, long: Option<f32>, alt: Option<u32>, info: Option<String>) -> Result<()>
    {
        let mut sql = "UPDATE sites SET ".to_string();

        if name.is_some()
        {
            sql.push_str(format!("name='{}',",name.unwrap()).as_str());
        }
        if lat.is_some()
        {
            sql.push_str(format!("lat={},",lat.unwrap()).as_str());
        }
        if long.is_some()
        {
            sql.push_str(format!("long={},",long.unwrap()).as_str());
        }
        if alt.is_some()
        {
            sql.push_str(format!("alt={},",alt.unwrap()).as_str());
        }
        if info.is_some()
        {
            sql.push_str(format!("info='{}',",info.unwrap()).as_str());
        }

        sql.pop();//Remove ','

        sql.push_str("WHERE site_id=?1");
        
        self.db_conn.execute(&sql, [id])?;

        Ok(())
    }

    pub fn get_sites(&self) -> Result<Vec<Site>>
    {
        let mut sites: Vec<Site> = Vec::new();
        let mut stmt = self.db_conn.prepare("SELECT * FROM sites")?;

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
            })?;

        for site in rows {
            if let Ok(f) = site {
                sites.push(f)
            }
        }

        Ok(sites)
    }

    pub fn delete_site(&self, id: i32) -> Result<()>
    {
        self
            .db_conn
            .execute("DELETE FROM sites WHERE site_id = ?1", [id])?;

        Ok(())
    }

    pub fn associate_tag(&self, tag_id: u32, flight_id: u32) -> Result<()>
    {
        self.db_conn.execute(
            "INSERT OR IGNORE INTO tag_asso (tag_id, flight_id)
                VALUES (?1, ?2)",
                (
                    tag_id,
                    flight_id,
                ),
            )?;
    
            Ok(())
    }

    pub fn store_tag(&self, tag: Tag) -> Result<()>
    {
        self.db_conn.execute(
        "INSERT OR IGNORE INTO tags (name)
            VALUES (?1)",
            (
                tag.name,
            ),
        )?;

        Ok(())
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>>
    {
        let mut tags: Vec<Tag> = Vec::new();
        let mut stmt = self.db_conn.prepare("SELECT * FROM tags")?;

        let rows = stmt
            .query_map([], |row| {
                Ok(Tag{
                    id: row.get(0).unwrap_or(255),
                    name: row.get(1).unwrap_or("None".to_string()),
                })
            })?;

        for tag in rows {
            if let Ok(f) = tag {
                tags.push(f)
            }
        }

        Ok(tags)
    }

    pub fn delete_tag(&self, id: i32) -> Result<()>
    {
        self.db_conn
            .execute("DELETE FROM tags WHERE tag_id = ?1", [id])?;

        Ok(())
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
