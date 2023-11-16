use std::{path::Path, fs};
use anyhow::{Result, bail};
use chrono::NaiveDateTime;
use crate::flight_track::FlightTrack;

use self::{flight_table::FlightTable, site_table::SiteTable, tag_table::TagTable, wing_table::WingTable};

pub mod flight_table;
pub mod site_table;
pub mod tag_table;
pub mod wing_table;

const DATABASE_PATH: &str = "./flight_database.db";

#[derive(Debug)]
pub struct IDListe
{
    list: Vec<u32>,
}

#[derive(Clone,Copy,Debug)]
pub struct FlightPoint{
    pub time: NaiveDateTime,
    pub lat: f32,
    pub long: f32,
    pub alt: u32,
    pub alt_gps: u32,
}

#[derive(Debug)]
pub struct FlightStatistic {
    pub duration: u32,
    pub tot_distance: u32,
    pub best_flight: Vec<FlightTable>,
    pub nb_flight: u32,
}

pub struct Logbook;

pub trait Statistic {
    fn statistic(&self) -> FlightStatistic;
}

impl Statistic for Vec<FlightTable> {
    fn statistic(&self) -> FlightStatistic {
        let mut duration: u32 = 0;
        let mut tot_distance: u32 = 0;
        let mut nb_flight: u32 = 0;
        let mut best_flight: Vec<FlightTable> = self.clone();

        for flight in self {
            duration += flight.duration;
            tot_distance += flight.distance;
            nb_flight += 1;
        }

        best_flight.sort_by(|a, b| b.distance.cmp(&a.distance));

        let index = if best_flight.len() > 3 {
            3
        } else {
            best_flight.len()
        };

        FlightStatistic {
            duration,
            tot_distance,
            best_flight: best_flight[..index].to_vec(),
            nb_flight,
        }
    }
}

impl ToString for IDListe {
    fn to_string(&self) -> String {       
        let mut str_ids = "(".to_string();

        for id in &self.list
        {
            str_ids.push_str(format!("{},",id).as_str());
        }

        str_ids.pop();//remove last ','
        str_ids.push(')');      

        str_ids
    }
}

impl Logbook {
    pub fn create() -> Result<()>
    {
        FlightTable::create()?;
        SiteTable::create()?;
        TagTable::create()?;
        WingTable::create()?;

        Ok(())
    }

    pub fn load(path: &Path) -> Result<Vec<FlightTable>>
    {
        let paths: &mut Vec<String> = &mut Vec::new();
        Self::search_igc(path, paths);
        let flights = &mut Vec::new();

        for path in paths 
        {
            let raw_igc: String = fs::read_to_string(path)?;
            match FlightTrack::new(&raw_igc)
            {
                Ok(t) => {
                    let sites = SiteTable::site_detection(t.takeoff, t.landing)?;
                    let wing = WingTable::get_default_wing()?;

                    let takeoff = match sites.0 {
                        None => 
                        {
                            let site = SiteTable { 
                                site_id:0,
                                name: "Unkown".to_string(),
                                lat: t.takeoff.lat,
                                long: t.takeoff.long,
                                alt: t.takeoff.alt,
                                info: "".to_string(),
                            };
                            SiteTable::store(site)?;
            
                            Some(SiteTable::last_site_id()?)
                        },
                        Some(s) => Some(s.site_id),
                    };

                    let landing = match sites.1 {
                        None => 
                        {
                            let site = SiteTable { 
                                site_id:0,
                                name: "Unkown".to_string(),
                                lat: t.landing.lat,
                                long: t.landing.long,
                                alt: t.landing.alt,
                                info: "".to_string(),
                            };
                            SiteTable::store(site)?;
            
                            Some(SiteTable::last_site_id()?)
                        },
                        Some(s) => Some(s.site_id),
                    };

                    flights.push(FlightTable { 
                        flight_id: 0, 
                        wing_id: wing.wing_id, 
                        takeoff_id: takeoff.unwrap_or(0), 
                        landing_id: landing.unwrap_or(0), 
                        hash: t.hash, 
                        date: t.date, 
                        duration: t.duration, 
                        distance: t.distance, 
                        points: None, 
                        raw_igc: Some(raw_igc)
                    });

                },
                Err(e) => {
                    dbg!(e.to_string());
                },
            }            
        }

        Ok(flights.to_vec())

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
}