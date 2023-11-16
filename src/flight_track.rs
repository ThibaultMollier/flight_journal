use std::{process::{Command, Stdio}, io::Write};

use crate::logbook::FlightPoint;
use self::igc_reader::IgcReader;
use anyhow::{Result, bail};
use chrono::NaiveDate;
use geoutils::{Location, Distance};
use serde_json::Value;

mod igc_reader;

const IGC_SCORER_PATH: &str = "./igc-xc-score.exe";

const EPSILON: f32 = 0.005;

const NB_POINT:usize = 15;
const HSPEED_THR:f64 = 3.0;// m/s - 11km/h
const VSPEED_THR:f64 = 0.6;// m/s

pub struct FlightTrack
{
    // track: Vec<FlightPoint>,
    // simplified_track: Vec<FlightPoint>,
    pub geojson: String,
    pub duration: u32,
    pub distance: u32,
    pub date: NaiveDate,
    pub takeoff: FlightPoint,
    pub landing: FlightPoint,
    pub hash: String,
}

impl FlightTrack {
    pub fn new(raw_igc: &String) -> Result<Self>
    {
        let mut igc_scorer = Command::new(IGC_SCORER_PATH)
            .arg("pipe=true")
            .arg("quiet=true")
            .arg("maxtime=5")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let temp = raw_igc.clone();

        let mut stdin = igc_scorer.stdin.take().expect("Failed to open stdin");
        std::thread::spawn(move || {
                stdin.write_all(temp.as_bytes()).expect("failed to write to stdin");
        });

        let igc = IgcReader::read(raw_igc)?;

        let takeoff_index = Self::flight_detection(&igc.track);
        let mut reversed_trace = igc.track.clone();
        reversed_trace.reverse();
        let landing_index = (&igc.track.len() - Self::flight_detection(&reversed_trace)).saturating_sub(1);

        let duration = igc.track[landing_index].time - igc.track[takeoff_index].time;

        // let simplified_track: Vec<FlightPoint> = Self::simplify(&igc.track[takeoff_index..landing_index].to_vec(), &EPSILON);
        // let distance: u32 = Self::total_distance(&simplified_track);

        let output = igc_scorer.wait_with_output().expect("Failed to read stdout");
        let geojson: Value = serde_json::from_slice(&output.stdout).unwrap();

        let code = geojson["properties"]["code"].to_string();

        let multiplier = if code == "\"tri\""{
            1.2
        }else if code == "\"fai\""
        {
            1.4
        }else{
            1.0
        };

        let distance = geojson["properties"]["score"].as_f64().unwrap()/multiplier;

        Ok(FlightTrack { 
            geojson: geojson.to_string(),
            duration: duration.num_minutes() as u32,
            distance: (distance*1000.0) as u32, 
            date: igc.date, 
            takeoff: igc.track[takeoff_index].clone(),
            landing: igc.track[landing_index].clone(),
            hash: igc.check
        })
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
}