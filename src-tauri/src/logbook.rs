use std::{path::Path, fs, process::{Command, Child, Stdio}, io::Write};
use anyhow::{Result, bail};
use chrono::NaiveDateTime;
use serde_json::Value;
use crate::flight_track::FlightTrack;

use self::{flight_table::FlightTable, site_table::SiteTable, tag_table::TagTable, wing_table::WingTable};

pub mod flight_table;
pub mod site_table;
pub mod tag_table;
pub mod wing_table;

const DATABASE_PATH: &str = "./flight_database.db";
const IGC_SCORER_PATH: &str = "./igc-xc-score.exe";
const SCORE_MAX_TIME: &str = "maxtime=5";

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

    pub fn load_and_store(path: &Path) -> Result<()>
    {
        let paths: &mut Vec<String> = &mut Vec::new();
        Self::search_igc(path, paths);

        for path in paths 
        {

            let (mut flight,scorer) = Logbook::load(Path::new(path))?;

            let (track,score,code) = Logbook::get_score(scorer)?;

            flight.track = Some(track.into());
            flight.score = score;
            flight.code = code;

            FlightTable::store(flight)?;
        }

        Ok(())
    }

    pub fn load(path: &Path) -> Result<(FlightTable,Child)>
    {
        let raw_igc: String = fs::read_to_string(path)?;
        let flight: Option<FlightTable>;

        let scorer = Logbook::score(&raw_igc)?;

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

                flight = Some(FlightTable { 
                                    flight_id: 0, 
                                    wing_id: wing.wing_id, 
                                    takeoff_id: takeoff.unwrap_or(0), 
                                    landing_id: landing.unwrap_or(0), 
                                    hash: t.hash, 
                                    date: t.date.format("%Y-%m-%d").to_string(), 
                                    duration: t.duration, 
                                    distance: t.distance, 
                                    score: 0,
                                    code: "".to_string(),
                                    track: None, 
                                    raw_igc: Some(raw_igc),
                                    profile: Some(t.profile.to_string()),
                                });

            },
            Err(e) => {
                bail!(e.to_string());
            },
        }

        match flight {
            Some(f) => Ok((f,scorer)),
            None => bail!("No flight"),
        }
    }

    pub fn get_score(scorer: Child) -> Result<(String,u32,String)>
    {
        let output = scorer.wait_with_output()?;
        let geojson: Value = serde_json::from_slice(&output.stdout)?;

        let code = geojson["properties"]["code"].to_string();
        let score = geojson["properties"]["score"].as_f64().unwrap();

        Ok((geojson.to_string(),(score*1000.0) as u32, code))
    }

    fn score(raw_igc: &String) -> Result<Child>
    {
        let mut igc_scorer = Command::new(IGC_SCORER_PATH)
            .arg("pipe=true")
            .arg("quiet=true")
            .arg(SCORE_MAX_TIME)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let temp = raw_igc.clone();

        let mut stdin = igc_scorer.stdin.take().expect("Failed to open stdin");
        std::thread::spawn(move || {
                stdin.write_all(temp.as_bytes()).expect("failed to write to stdin");
        });

        Ok(igc_scorer)
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