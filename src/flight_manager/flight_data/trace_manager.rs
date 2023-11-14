use chrono::{DateTime, Datelike, LocalResult, NaiveDate, TimeZone, Utc};
use geoutils::{Distance, Location};
use anyhow::{Result, bail};

const IGC_HEADER: &str = "H";
const IGC_DATE: &str = "FDTE";

const IGC_RECORD: &str = "B";
const IGC_CHECK: &str = "G";

const EPSILON: f32 = 0.005;

const NB_POINT:usize = 15;
const HSPEED_THR:f64 = 3.0;// m/s - 11km/h
const VSPEED_THR:f64 = 0.6;// m/s

#[derive(Clone, Debug)]
pub struct FlightPoint {
    pub time: DateTime<Utc>,
    pub lat: f32,
    pub long: f32,
    pub alt: u32,
    pub alt_gps: u32,
}

#[derive(Clone, Debug)]
pub struct FlightTrace {
    pub check: String,
    pub raw_igc: String,
    pub date: NaiveDate,
    pub simplified_trace: Vec<FlightPoint>,
    pub trace: Vec<FlightPoint>,
}

impl FlightTrace {
    pub fn new(raw_igc: String) -> Self {
        let mut trace: Vec<FlightPoint> = Vec::new();
        let mut date: NaiveDate = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let igc_lines = raw_igc.lines();
        let mut check = "".to_string();

        for line in igc_lines {
            if line.chars().nth(0) == IGC_RECORD.chars().nth(0) {
                trace.push(Self::process_record(line, &date));
            } else if line.chars().nth(0) == IGC_HEADER.chars().nth(0) {
                if line[IGC_HEADER.len()..IGC_HEADER.len() + IGC_DATE.len()] == *IGC_DATE {
                    if let Ok(d) = Self::process_date(line) {
                        date = d
                    }
                }
            } else if line.chars().nth(0) == IGC_CHECK.chars().nth(0) {
                check = line[IGC_CHECK.len()..].to_string();
            }
        }

        let takeoff_index = Self::flight_detection(&trace);
        let mut reversed_trace = trace.clone();
        // dbg!(&trace.get(takeoff_index).unwrap().time);
        reversed_trace.reverse();
        let landing_index = trace.len() - Self::flight_detection(&reversed_trace) - 1;

        // dbg!(&trace.get(landing_index).unwrap().time);

        FlightTrace {
            check,
            raw_igc,
            date,
            simplified_trace: Self::douglas_peucker(&trace[takeoff_index..landing_index].to_vec(), &EPSILON),
            trace,
        }
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

    pub fn total_distance(&self) -> u32 {
        let mut dist: f64 = 0.0;

        if self.simplified_trace.len() < 2
        {
            return 0;
        }

        for i in 0..self.simplified_trace.len() - 1 {
            dist += Location::new(self.simplified_trace[i].lat, self.simplified_trace[i].long)
                .distance_to(&Location::new(
                    self.simplified_trace[i + 1].lat,
                    self.simplified_trace[i + 1].long,
                ))
                .unwrap_or(Distance::from_meters(0))
                .meters();
        }

        dist as u32
    }

    pub fn flight_duration(&self) -> u32 {
        if self.simplified_trace.len() < 2
        {
            return 0;
        }

        let duration: chrono::Duration =
            self.simplified_trace.last().unwrap().time - self.simplified_trace.get(0).unwrap().time;

        duration.num_minutes() as u32
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

    pub fn douglas_peucker(pointlist: &Vec<FlightPoint>, epsilon: &f32) -> Vec<FlightPoint> {
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
                Self::douglas_peucker(&pointlist.get(..index as usize).unwrap().to_vec(), epsilon);
            let mut res2 =
                Self::douglas_peucker(&pointlist.get(index as usize..).unwrap().to_vec(), epsilon);

            result = res1;
            result.append(&mut res2);
        } else {
            result.push(pointlist[0].clone());
            result.push(pointlist[end - 1].clone());
        }

        result
    }

    pub fn triangle(pointlist: &Vec<FlightPoint>) -> (Vec<FlightPoint>, f64) {
        let mut index: usize = 0;
        let mut dmax: f64 = 0.0;
        let mut p1: FlightPoint = FlightPoint {
            time: Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0).unwrap(),
            lat: 0.0,
            long: 0.0,
            alt: 0,
            alt_gps: 0,
        };
        let mut p2: FlightPoint = p1.clone();
        let mut p3: FlightPoint = p1.clone();

        for pt1 in pointlist {
            index += 1;
            if index < (pointlist.len() - 1) {
                for pt2 in pointlist.get(index..).unwrap() {
                    let loc1 = Location::new(pt1.lat, pt1.long);
                    let loc2 = Location::new(pt2.lat, pt2.long);
                    let d1: f64 = loc1
                        .distance_to(&loc2)
                        .unwrap_or(Distance::from_meters(0))
                        .meters();

                    if d1 > dmax / 4.0
                    //To limitate computation time
                    {
                        for pt3 in pointlist.get(index + 1..).unwrap() {
                            let loc3 = Location::new(pt3.lat, pt3.long);
                            let d2: f64 = loc1
                                .distance_to(&loc3)
                                .unwrap_or(Distance::from_meters(0))
                                .meters();
                            let d3: f64 = loc2
                                .distance_to(&loc3)
                                .unwrap_or(Distance::from_meters(0))
                                .meters();

                            if (d1 + d2 + d3) > dmax {
                                dmax = d1 + d2 + d3;
                                p1 = pt1.clone();
                                p2 = pt2.clone();
                                p3 = pt3.clone();
                            }
                        }
                    }
                }
            }
        }

        // dbg!(dmax);
        (vec![p1, p2, p3], dmax)
    }

    fn _simplify(pointlist: &Vec<FlightPoint>,n: u32) -> Vec<FlightPoint>
    {
        let mut dmax: f64 = 0.0;
        let mut index: u32 = 0;
        let mut cpt: u32 = 1;
        let end: usize = pointlist.len();
        let mut result: Vec<FlightPoint> = Vec::new();

        if end < 3
        {
            return pointlist.to_vec();
        }

        for pt in pointlist.get(1..end).unwrap()
        {
            let d1: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist[0].lat, pointlist[0].long)).unwrap().meters();

            let d2: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist.last().unwrap().lat, pointlist.last().unwrap().long)).unwrap().meters();

            if (d1 + d2) > dmax
            {
                dmax = d1 + d2;
                index = cpt;
            }
            cpt += 1;
        }

        if n > 0 && index > 1
        {
            let n = n - 1;

            let res1 = Self::_simplify(&pointlist.get(..index as usize).unwrap().to_vec(), n);
            let mut res2 = Self::_simplify(&pointlist.get(index as usize..).unwrap().to_vec(), n);

            result = res1;
            result.append(&mut res2);
        }else {
            result.push(pointlist[0].clone());
            result.push(pointlist.last().unwrap().clone());
        }

        return result;
    }

    fn to_decimal(degree: u32, minute: f32, snew: char) -> f32 {
        let mut decimal: f32 = minute / 60.0;
        decimal += degree as f32;

        if snew == 'S' || snew == 'W' {
            decimal = -decimal;
        }

        decimal
    }

    fn process_date(line: &str) -> Result<NaiveDate> {
        let mut c = line.chars().peekable();
        //Search the first numeric value on the line if the file doesn't respect the format
        while !c.next_if(|&x| !x.is_numeric()).unwrap_or('0').is_numeric() {}

        let day: u32 = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
            + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);
        let month: u32 = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
            + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);
        let year: i32 = (2000
            + c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
            + c.next().unwrap_or('0').to_digit(10).unwrap_or(0))
        .try_into()
        .unwrap_or(0);

        let date = NaiveDate::from_ymd_opt(year, month, day);

        match date {
            None => bail!("Igc date extract failed"),
            Some(d) => Ok(d),
        }
    }

    fn process_record(line: &str, date: &NaiveDate) -> FlightPoint {
        let mut c = line[IGC_RECORD.len()..].chars();
        let hour: u32 = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
            + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);
        let min: u32 = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
            + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);
        let sec: u32 = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
            + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);

        let lat: f32 = {
            let degree: u32 = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);
            let minute: u32 = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);
            let minute: f32 = (minute as f32)
                + (c.next().unwrap_or('0').to_digit(10).unwrap_or(0) as f32) / 10.0
                + (c.next().unwrap_or('0').to_digit(10).unwrap_or(0) as f32) / 100.0
                + (c.next().unwrap_or('0').to_digit(10).unwrap_or(0) as f32) / 1000.0;

            let snew: char = c.next().unwrap_or('0');

            Self::to_decimal(degree, minute, snew)
        };

        let long = {
            let degree = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 100
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);
            let minute = c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0);
            let minute = (minute as f32)
                + (c.next().unwrap_or('0').to_digit(10).unwrap_or(0) as f32) / 10.0
                + (c.next().unwrap_or('0').to_digit(10).unwrap_or(0) as f32) / 100.0
                + (c.next().unwrap_or('0').to_digit(10).unwrap_or(0) as f32) / 1000.0;

            let snew = c.next().unwrap_or('0');

            Self::to_decimal(degree, minute, snew)
        };

        let _validity = c.next().unwrap_or('0') == 'A';
        let alt = {
            c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10000
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 1000
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 100
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0)
        }; //Barometric

        let alt_gps = {
            c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10000
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 1000
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 100
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0) * 10
                + c.next().unwrap_or('0').to_digit(10).unwrap_or(0)
        };

        let time: DateTime<Utc> =
            match Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), hour, min, sec) {
                LocalResult::None => Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0).unwrap(),
                LocalResult::Single(t) => t,
                LocalResult::Ambiguous(_, t) => t,
            };

        FlightPoint {
            time,
            long,
            lat,
            alt,
            alt_gps,
        }
    }
}
