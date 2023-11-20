use std::{io::Write, fs};

use crate::logbook::FlightPoint;
use self::igc_reader::IgcReader;
use anyhow::{Result, bail};
use chrono::{NaiveDate, NaiveDateTime};
use geoutils::{Location, Distance};

mod igc_reader;

const EPSILON: f32 = 0.00001;

const NB_POINT:usize = 15;
const HSPEED_THR:f64 = 3.0;// m/s - 11km/h
const VSPEED_THR:f64 = 0.6;// m/s

pub struct FlightProfile
{
    pub points: Vec<FlightProfilePoint>,
}

pub struct FlightProfilePoint
{
    time: NaiveDateTime,
    alt: u32,
    speed: u32, // m/s
    vario: f32, // m/s
    lat: f32,
    lng: f32,
}

pub struct FlightTrack
{
    // track: Vec<FlightPoint>,
    // simplified_track: Vec<FlightPoint>,
    // pub geojson: String,
    pub profile: FlightProfile,
    pub duration: u32,
    pub distance: u32,
    pub date: NaiveDate,
    pub takeoff: FlightPoint,
    pub landing: FlightPoint,
    pub hash: String,
}

impl ToString for FlightProfile {
    fn to_string(&self) -> String {
        let mut csv = String::new();
        let mut ts_col = String::new();
        let mut alt_col = String::new();
        let mut speed_col = String::new();
        let mut vario_col = String::new();
        let mut lat_col = String::new();
        let mut lng_col = String::new();

        for pt in &self.points
        {
            ts_col.push_str(format!("{},",pt.time.timestamp()).as_str());
            alt_col.push_str(format!("{},",pt.alt).as_str());
            speed_col.push_str(format!("{},",pt.speed).as_str());
            vario_col.push_str(format!("{},",pt.vario).as_str());
            lat_col.push_str(format!("{},",pt.lat).as_str());
            lng_col.push_str(format!("{},",pt.lng).as_str());
            // csv.push_str(format!("{},{},{},{}\n",pt.time.timestamp(),pt.alt,pt.speed,pt.vario).as_str());
        }

        ts_col.push_str("\n");
        alt_col.push_str("\n");
        speed_col.push_str("\n");
        vario_col.push_str("\n");
        lat_col.push_str("\n");
        lng_col.push_str("\n");

        csv.push_str(&ts_col);
        csv.push_str(&alt_col);
        csv.push_str(&speed_col);
        csv.push_str(&vario_col);
        csv.push_str(&lat_col);
        csv.push_str(&lng_col);

        csv
    }
}

impl FlightTrack {
    pub fn new(raw_igc: &String) -> Result<Self>
    {
        let igc = IgcReader::read(raw_igc)?;

        let takeoff_index = Self::flight_detection(&igc.track);
        let mut reversed_trace = igc.track.clone();
        reversed_trace.reverse();
        let landing_index = (&igc.track.len() - Self::flight_detection(&reversed_trace)).saturating_sub(1);

        let duration = igc.track[landing_index].time - igc.track[takeoff_index].time;

        let simplified_track: Vec<FlightPoint> = Self::simplify(&igc.track[takeoff_index..landing_index].to_vec(), &EPSILON);
        let distance: u32 = Self::total_distance(&simplified_track);

        Self::_to_file(&simplified_track);
        println!("{} {}",simplified_track.len(),igc.track.len());

        Ok(FlightTrack { 
            profile: Self::flight_profile(&simplified_track),
            duration: duration.num_minutes() as u32,
            distance, 
            date: igc.date, 
            takeoff: igc.track[takeoff_index].clone(),
            landing: igc.track[landing_index].clone(),
            hash: igc.check
        })
    }

    fn flight_profile(trace: &Vec<FlightPoint>) -> FlightProfile
    {
        let mut profile: FlightProfile = FlightProfile { points: Vec::new() };

        for i in 1..trace.len()
        {
            let delta = (trace[i].time - trace[i - 1].time).num_seconds() as u32;

            let vario = (trace[i].alt as i32 - trace[i - 1].alt as i32) as f32 / delta as f32;
            let pt1 = Location::new(trace[i].lat,trace[i].long);
            let pt2 = Location::new(trace[i - 1].lat,trace[i - 1].long);

            let speed = pt1.distance_to(&pt2).unwrap().meters() / delta as f64;
            profile.points.push(FlightProfilePoint { 
                time: trace[i].time, 
                alt: trace[i].alt, 
                speed: speed as u32, 
                vario,
                lat: pt1.latitude() as f32,
                lng: pt1.longitude() as f32,
            });
        }

        profile
    }

    fn flight_detection(trace: &Vec<FlightPoint>) -> usize {
        let mut index: usize = 0;

        if trace.len() < 5
        {
            return index;
        }
        
        for i in 0..(trace.len() - 2)
        {
            let mut vspeed: f64 = 0.0;
            let mut hspeed: f64 = 0.0;
            for j in 0..NB_POINT
            {
                vspeed += trace[i + j].alt as f64 - trace[i + j + 1].alt as f64;
                let loc1 = Location::new(trace[i + j].lat,trace[i + j].long);
                let loc2 = Location::new(trace[i + j + 1].lat,trace[i + j + 1].long);
                hspeed += loc1.distance_to(&loc2).unwrap().meters();
            }

            vspeed = vspeed/(NB_POINT as f64);
            hspeed = hspeed/(NB_POINT as f64);

            // println!("{} {}",vspeed,hspeed);

            if vspeed < -VSPEED_THR || vspeed > VSPEED_THR || hspeed > HSPEED_THR {
                index = i;
                break;
            }
        }

        index
    }

    fn total_distance(track: &Vec<FlightPoint>) -> u32 {
        let mut dist: f64 = 0.0;

        if track.len() < 2
        {
            return 0;
        }

        for i in 0..track.len() - 1 {
            dist += Location::new(track[i].lat, track[i].long)
                .distance_to(&Location::new(
                    track[i + 1].lat,
                    track[i + 1].long,
                ))
                .unwrap_or(Distance::from_meters(0))
                .meters();
        }

        dist as u32
    }

    fn magnitude(p1: &FlightPoint, p2: &FlightPoint) -> f32 {
        let line = FlightPoint {
            lat: p2.lat - p1.lat,
            long: p2.long - p1.long,
            time: p2.time,
            alt: 0,
            alt_gps: 0,
        };

        f32::sqrt(line.lat * line.lat + line.long * line.long)
    }

    fn distance_point_line(
        p: &FlightPoint,
        line_start: &FlightPoint,
        line_end: &FlightPoint,
    ) -> f32 {
        let linemag = Self::magnitude(line_start, line_end);

        let u = (((p.lat - line_start.lat) * (line_end.lat - line_start.lat))
            + ((p.long - line_start.long) * (line_end.long - line_start.long)))
            / (linemag * linemag);

        // if !(0.0..=1.0).contains(&u) {
        //     return 0.0;
        // }

        let intersection = FlightPoint {
            lat: line_start.lat + u * (line_end.lat - line_start.lat),
            long: line_start.long + u * (line_end.long - line_start.long),
            time: line_start.time,
            alt: 0,
            alt_gps: 0,
        };

        Self::magnitude(p, &intersection)
    }

    pub fn simplify(pointlist: &Vec<FlightPoint>, epsilon: &f32) -> Vec<FlightPoint> {
        let mut dmax: f32 = 0.0;
        let mut index: u32 = 0;
        let mut cpt: u32 = 1;
        let end: usize = pointlist.len();
        let mut result = Vec::new();

        if end < 3 {
            return pointlist.to_vec();
        }

        for pt in match pointlist.get(1..end) {
            None => return pointlist.to_vec(),
            Some(p) => p.to_vec(),
        } {
            let d = Self::distance_point_line(&pt, &pointlist[0], pointlist.last().unwrap());

            if d > dmax {
                dmax = d;
                index = cpt;
            }
            cpt += 1;
        }

        if (index as usize) > end {
            return pointlist.to_vec();
        }

        if dmax > *epsilon {
            let res1 =
                Self::simplify(&pointlist.get(..index as usize).unwrap().to_vec(), epsilon);
            let mut res2 =
                Self::simplify(&pointlist.get(index as usize..).unwrap().to_vec(), epsilon);

            result = res1;
            result.append(&mut res2);
        } else {
            result.push(pointlist[0].clone());
            result.push(pointlist[end - 1].clone());
        }

        result
    }

    fn _to_file(trace: &Vec<FlightPoint>) {
        let path = "./trace";
    
        let mut output = fs::File::create(path).unwrap();
    
        for pt in trace {
            writeln!(output, "{},{}", pt.lat, pt.long).unwrap();
        }
    }
}

