use chrono::{Utc, TimeZone, Datelike};
use geoutils::Location;
use std::fs::File;
use std::io::Write;

use crate::flight_manager::flight_data::flight_simplify::FlightSimplify;

use self::igc_reader::IgcReader;

mod flight_simplify;
mod igc_reader;

#[derive(Debug)]
pub struct FlightData{
    pub date: String,
    pub duration: i64,
    pub distance: u32,
}

#[derive(Clone, Debug)]
pub struct FlightPoint{
    time: chrono::DateTime<Utc>,
    lat: f32,
    long: f32,
    alt: u32,
    alt_gps: u32,
}

impl FlightData {
    pub fn compute(igc: &String) -> Self
    {
        let igc_data = IgcReader::read(igc);

        let duration = igc_data.trace.last().unwrap().time - igc_data.trace.get(0).unwrap().time;

        let dist = FlightData::compute_distance(&igc_data.trace);

        let flightdata: FlightData = FlightData {
            date: format!("{}-{:02}-{:02}",igc_data.date.year(),igc_data.date.month(),igc_data.date.day()),
            duration: duration.num_minutes(),
            distance: dist.floor() as u32,
        };

        return flightdata;
    }

    fn to_file(trace: &Vec<FlightPoint>)
    {
        let path = "./trace";

        let mut output = File::create(path).unwrap();

        for pt in trace
        {
            write!(output, "{},{}\n",pt.lat,pt.long).unwrap();
        }
        
    }

    fn compute_distance(trace: &Vec<FlightPoint>) -> f64
    {
        let mut dist: f64 = 0.0;
        let epsilon: f32 = 0.02;

        // let simplified_trace: Vec<FlightPoint> = FlightSimplify::simplify(trace, 6);

        let simplified_trace: Vec<FlightPoint> = FlightSimplify::douglas_peucker(trace, &epsilon);

        println!("{}",simplified_trace.len());

        for i in 0..simplified_trace.len()-1
        {
            dist += Location::new(simplified_trace[i].lat, simplified_trace[i].long)
            .distance_to(&Location::new(simplified_trace[i+1].lat, simplified_trace[i+1].long)).unwrap().meters(); 
        }

        // let (simplified_trace,dist) = FlightSimplify::triangle(&simplified_trace);
        
        FlightData::to_file(&simplified_trace);

        println!("\t{:3.1}km",dist/1000.0);

        return dist;
    }



    
}

