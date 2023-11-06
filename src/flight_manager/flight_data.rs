use std::{path::Path, fs};
use chrono::{Utc, TimeZone, Datelike, NaiveDate};
use geoutils::{Location, Distance};
use self::trace_manager::{FlightTrace, FlightPoint};
use crate::flight_manager::Error;

pub mod trace_manager;

#[derive(Clone)]
pub struct FlightData{
    pub id:         Option<u32>,
    pub hash:       String,
    pub duration:   u32,
    pub date:       NaiveDate,
    pub distance:   u32,
    pub takeoff:    Option<String>,
    pub landing:    Option<String>,
    pub tags:       Option<String>,
    pub points:     Option<Vec<FlightPoint>>,
    pub trace:      Option<FlightTrace>,
    pub wing:       String,
}

impl FlightData {
    pub fn from_igc<P: AsRef<Path>>(path: P) -> Result<FlightData,Error>
    {
        let raw_igc: String = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Err(Error::FileErr),
        };
        let trace: FlightTrace = FlightTrace::new(raw_igc);
        let mut dist: f64 = 0.0;

        let duration: chrono::Duration = trace.trace.last().unwrap().time - trace.trace.get(0).unwrap().time;

        for i in 0..trace.simplified_trace.len()-1
        {
            dist += Location::new(trace.simplified_trace[i].lat, trace.simplified_trace[i].long)
            .distance_to(&Location::new(trace.simplified_trace[i+1].lat, trace.simplified_trace[i+1].long)).unwrap_or(Distance::from_meters(0)).meters(); 
        }

        Ok(FlightData{
            id: None,
            hash: trace.check.clone(),
            duration: duration.num_minutes() as u32,
            date: trace.date,
            distance: dist as u32,
            takeoff: None,
            landing: None,
            tags: None,
            points: None,
            trace: Some(trace),
            wing: "".to_string(),
        })

    }
}