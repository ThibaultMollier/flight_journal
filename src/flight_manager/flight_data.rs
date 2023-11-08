use chrono::NaiveDate;
use std::io::Write;
use std::{fs, path::Path};
// use geoutils::{Location, Distance};
use self::trace_manager::{FlightPoint, FlightTrace};
use crate::flight_manager::Error;

pub mod trace_manager;

pub struct Wing
{
    pub id: u32,
    pub name: String,
    pub info: String,
}

pub struct Site
{
    pub id: u32,
    pub name: String,
    pub lat: f32,
    pub long: f32,
    pub info: String,
}

#[derive(Debug)]
pub struct FlightStatistic {
    pub duration: u32,
    pub tot_distance: u32,
    pub best_flight: Vec<FlightData>,
    pub nb_flight: u32,
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

#[derive(Debug, Clone)]
pub struct FlightData {
    pub id: Option<u32>,
    pub hash: String,
    pub duration: u32,
    pub date: NaiveDate,
    pub distance: u32,
    pub takeoff: Option<String>,
    pub landing: Option<String>,
    pub tags: Option<String>,
    pub points: Option<Vec<FlightPoint>>,
    pub trace: Option<FlightTrace>,
    pub wing: String,
}

impl FlightData {
    pub fn from_igc<P: AsRef<Path>>(path: P) -> Result<FlightData, Error> {
        let raw_igc: String = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Err(Error::FileErr),
        };
        let trace: FlightTrace = FlightTrace::new(raw_igc);

        Ok(FlightData {
            id: None,
            hash: trace.check.clone(),
            duration: trace.flight_duration(),
            date: trace.date,
            distance: trace.total_distance(),
            takeoff: None,
            landing: None,
            tags: None,
            points: None,
            trace: Some(trace),
            wing: "".to_string(),
        })
    }

    fn to_file(trace: &Vec<FlightPoint>) {
        let path = "./trace";

        let mut output = fs::File::create(path).unwrap();

        for pt in trace {
            writeln!(output, "{},{}", pt.lat, pt.long).unwrap();
        }
    }
}
