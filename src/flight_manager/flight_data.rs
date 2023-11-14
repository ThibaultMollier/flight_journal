use chrono::NaiveDate;
use geoutils::Location;
use std::{fs, path::Path};
use self::trace_manager::{FlightPoint, FlightTrace};
use std::io::Write;
use super::FlightManager;
use anyhow::{Result, bail};

pub mod trace_manager;

const DISTANCE_DETECTION: f64 = 200.0;

pub struct Tag
{
    pub id: u32,
    pub name: String,
}
pub struct Wing
{
    pub id: u32,
    pub name: String,
    pub info: String,
    pub default: Option<bool>,
}

#[derive(Debug,Clone)]
pub struct Site
{
    pub id: u32,
    pub name: String,
    pub lat: f32,
    pub long: f32,
    pub alt: u32,
    pub info: String,
}

#[derive(Debug)]
pub struct FlightStatistic {
    pub duration: u32,
    pub tot_distance: u32,
    pub best_flight: Vec<FlightData>,
    pub nb_flight: u32,
}

#[derive(Debug, Clone)]
pub struct FlightData {
    pub id: Option<u32>,
    pub hash: String,
    pub duration: u32,
    pub date: NaiveDate,
    pub distance: u32,
    pub takeoff: Option<u32>,
    pub landing: Option<u32>,
    pub points: Option<Vec<FlightPoint>>,
    pub trace: Option<FlightTrace>,
    pub wing: u32,
}

pub trait Statistic {
    fn statistic(&self) -> FlightStatistic;
}

impl Statistic for Vec<FlightData> {
    fn statistic(&self) -> FlightStatistic {
        let mut duration: u32 = 0;
        let mut tot_distance: u32 = 0;
        let mut nb_flight: u32 = 0;
        let mut best_flight: Vec<FlightData> = self.clone();

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

pub trait FlightCompute {
    fn from_igc<P: AsRef<Path>>(&self, path: P) -> Result<FlightData>;
    fn site_detection(&self,trace: &FlightTrace) -> Result<(Option<Site>, Option<Site>)>;
}

impl FlightCompute for FlightManager {
    fn from_igc<P: AsRef<Path>>(&self, path: P) -> Result<FlightData> {
        let raw_igc: String = fs::read_to_string(path)?;
        let trace: FlightTrace = FlightTrace::new(raw_igc);

        if trace.trace.len() < 5
        {
            bail!("No trace available");
        }

        let sites = self.site_detection(&trace).unwrap_or((None,None));

        let takeoff = match sites.0 {
            None => 
            {
                let site = Site { 
                    id: 0, 
                    name: "Unknown".to_string(), 
                    lat: trace.trace[0].lat, 
                    long: trace.trace[0].long, 
                    alt: trace.trace[0].alt, 
                    info: "".to_string() 
                };
                self.store_site(site).unwrap_or(());

                Some(self.last_site_id()?)
            },
            Some(s) => Some(s.id),
        };
        let landing = match sites.1 {
            None => {
                let site = Site { 
                    id: 0, 
                    name: "Unknown".to_string(), 
                    lat: trace.trace.last().unwrap().lat, 
                    long: trace.trace.last().unwrap().long, 
                    alt: trace.trace.last().unwrap().alt, 
                    info: "".to_string() 
                };
                self.store_site(site).unwrap_or(());

                Some(self.last_site_id()?)
            },
            Some(s) => Some(s.id),
        };

        Ok(FlightData {
            id: None,
            hash: trace.check.clone(),
            duration: trace.flight_duration(),
            date: trace.date,
            distance: trace.total_distance(),
            takeoff,
            landing,
            points: None,
            trace: Some(trace),
            wing: 0,
        })
    }

    fn site_detection(&self,trace: &FlightTrace) -> Result<(Option<Site>, Option<Site>)>
    {
        let sites = self.get_sites()?;
        let mut res_takeoff: Option<Site> = None;
        let mut res_landing: Option<Site> = None;

        if trace.trace.len() < 5
        {
            bail!("Incorrect trace length");
        }

        let takeoff = Location::new(trace.trace[0].lat,trace.trace[0].long);
        let landing = Location::new(trace.trace.last().unwrap().lat,trace.trace.last().unwrap().long);

        for site in sites
        {
            let site_loc = Location::new(site.lat,site.long);
            let d = takeoff.distance_to(&site_loc).unwrap().meters();

            if d < DISTANCE_DETECTION
            {
                res_takeoff = Some(site.clone());
            }

            let d = landing.distance_to(&site_loc).unwrap().meters();
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
}


fn _to_file(trace: &Vec<FlightPoint>) {
    let path = "./trace";

    let mut output = fs::File::create(path).unwrap();

    for pt in trace {
        writeln!(output, "{},{}", pt.lat, pt.long).unwrap();
    }
}
