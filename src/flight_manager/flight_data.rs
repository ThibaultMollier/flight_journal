use chrono::{Utc, TimeZone, Datelike};
use geoutils::Location;

const EPSILON_COEF:f64 = 1.5;

const IGC_HEADER: &str = "H";
const IGC_DATE: &str = "FDTE";

const IGC_RECORD: &str = "B";

#[derive(Debug)]
pub struct FlightData{
    pub date: String,
    pub duration: i64,
    pub distance: u32,
}

#[derive(Clone, Debug)]
struct FlightPoint{
    time: chrono::DateTime<Utc>,
    lat: f32,
    long: f32,
    alt: u32,
    alt_gps: u32,
}

impl FlightData {
    pub fn compute(igc: &String) -> Self
    {
        let igc_lines = igc.lines();
        let mut trace: Vec<FlightPoint> = Vec::new();
        let mut date: chrono::DateTime<Utc> = Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0).unwrap();

        for line in igc_lines
        {
            if line.chars().nth(0) == IGC_RECORD.chars().nth(0)
            {
                trace.push(FlightData::process_record(line, &date));
            }else if line.chars().nth(0) == IGC_HEADER.chars().nth(0)
            {
                if line[IGC_HEADER.len()..IGC_HEADER.len()+IGC_DATE.len()] == *IGC_DATE
                {
                    date = FlightData::process_date(line);
                }
            }
        }

        let duration = trace.last().unwrap().time - trace.get(0).unwrap().time;

        let dist = FlightData::compute_distance(&trace);

        let flightdata: FlightData = FlightData {
            date: format!("{}-{:02}-{:02}",date.year(),date.month(),date.day()),
            duration: duration.num_minutes(),
            distance: dist.floor() as u32,
        };

        return flightdata;
    }

    fn process_date(line: &str) -> chrono::DateTime<Utc>
    {
        let mut c = line.chars().peekable();
        //Search the first numeric value on the line if the file doesn't respect the format
        while !c.next_if(|&x| !x.is_numeric()).unwrap_or('0').is_numeric() {}

        let day: u32    = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
        let month: u32  = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
        let year: i32   = (2000 + c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap()).try_into().unwrap();

        return Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
    }

    fn compute_distance(trace: &Vec<FlightPoint>) -> f64
    {
        let mut dist: f64 = 0.0;
        let epsilon: f64 = FlightData::compute_epsilon(trace);
        let simplified_trace: Vec<FlightPoint> = FlightData::douglas_peucker(trace, &epsilon);


        for i in 0..simplified_trace.len()-1
        {
            dist += Location::new(simplified_trace[i].lat, simplified_trace[i].long)
            .distance_to(&Location::new(simplified_trace[i+1].lat, simplified_trace[i+1].long)).unwrap().meters(); 
        }

        return dist;
    }

    fn process_record(line: &str, date: &chrono::DateTime<Utc>) -> FlightPoint
    {  
        let mut c = line[IGC_RECORD.len()..].chars();
        let hour: u32   = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
        let min: u32    = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
        let sec: u32    = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();

        let lat: f32 = {
            let degree: u32 = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
            let minute: u32 = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
            let minute: f32 = (minute as f32) +
                (c.next().unwrap().to_digit(10).unwrap() as f32)/10.0 + 
                (c.next().unwrap().to_digit(10).unwrap() as f32)/100.0 + 
                (c.next().unwrap().to_digit(10).unwrap() as f32)/1000.0;
    
            let snew: char = c.next().unwrap();
    
            FlightData::to_decimal(degree, minute, snew)
        };
        
        let long = {
            let degree = c.next().unwrap().to_digit(10).unwrap()*100 + c.next().unwrap().to_digit(10).unwrap()*10 +
                c.next().unwrap().to_digit(10).unwrap();
            let minute = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
            let minute = (minute as f32) +
                (c.next().unwrap().to_digit(10).unwrap() as f32)/10.0 + 
                (c.next().unwrap().to_digit(10).unwrap() as f32)/100.0 + 
                (c.next().unwrap().to_digit(10).unwrap() as f32)/1000.0;

            let snew = c.next().unwrap();

            FlightData::to_decimal(degree, minute, snew)
        };

        let _validity = c.next().unwrap() == 'A';
        let alt = {
            c.next().unwrap().to_digit(10).unwrap()*10000 + c.next().unwrap().to_digit(10).unwrap()*1000 +
            c.next().unwrap().to_digit(10).unwrap()*100 + c.next().unwrap().to_digit(10).unwrap()*10 +
            c.next().unwrap().to_digit(10).unwrap()
        }; //Barometric

        let alt_gps = {
            c.next().unwrap().to_digit(10).unwrap()*10000 + c.next().unwrap().to_digit(10).unwrap()*1000 +
            c.next().unwrap().to_digit(10).unwrap()*100 + c.next().unwrap().to_digit(10).unwrap()*10 +
            c.next().unwrap().to_digit(10).unwrap()
        }; 

        FlightPoint { 
            time: Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), hour, min, sec).unwrap(),
            long,
            lat,
            alt,
            alt_gps,
        }
    }

    fn to_decimal(degree:u32, minute:f32,snew: char) -> f32
    {
        let mut decimal:f32 = minute/60.0;
        decimal += degree as f32;

        if snew == 'S' || snew == 'W'
        {
            decimal = -decimal;
        }

        return decimal;
    }

    fn compute_epsilon(pointlist: &Vec<FlightPoint>) -> f64
    {
        let mut dmax: f64 = 0.0;
        let end: usize = pointlist.len();

        for pt in pointlist.get(1..end).unwrap()
        {
            let d1: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist[0].lat, pointlist[0].long)).unwrap().meters();

            let d2: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist[end - 1].lat, pointlist[end - 1].long)).unwrap().meters();

            if (d1 + d2) > dmax
            {
                dmax = d1 + d2;
            }
        }

        return dmax/EPSILON_COEF;
    }

    fn douglas_peucker(pointlist: &Vec<FlightPoint>,epsilon: &f64) -> Vec<FlightPoint>
    {
        let mut dmax: f64 = 0.0;
        let mut index: u32 = 0;
        let mut cpt: u32 = 1;
        let end: usize = pointlist.len();

        if end <= 1
        {
            return Vec::new();
        }

        for pt in pointlist.get(1..end).unwrap()
        {
            let d1: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist[0].lat, pointlist[0].long)).unwrap().meters();

            let d2: f64 = Location::new(pt.lat, pt.long)
                .distance_to(&Location::new(pointlist[end - 1].lat, pointlist[end - 1].long)).unwrap().meters();

            if (d1 + d2) > dmax
            {
                dmax = d1 + d2;
                index = cpt;
            }
            cpt += 1;
        }

        let mut result = Vec::new();

        if dmax > *epsilon
        {
            let res1 = FlightData::douglas_peucker(&pointlist.get(1..index as usize).unwrap().to_vec(), epsilon);
            let mut res2 = FlightData::douglas_peucker(&pointlist.get(index as usize..).unwrap().to_vec(), epsilon);
        
            result = res1;
            result.append(&mut res2);
        }else {
            result.push(pointlist[0].clone());
            result.push(pointlist[end - 1].clone());
        }

        return result;
    }
}

// function DouglasPeucker(PointList[], epsilon)
//     # Find the point with the maximum distance
//     dmax = 0
//     index = 0
//     end = length(PointList)
//     for i = 2 to (end - 1) {
//         d = perpendicularDistance(PointList[i], Line(PointList[1], PointList[end])) 
//         if (d > dmax) {
//             index = i
//             dmax = d
//         }
//     }

//     ResultList[] = empty;

//     # If max distance is greater than epsilon, recursively simplify
//     if (dmax > epsilon) {
//         # Recursive call
//         recResults1[] = DouglasPeucker(PointList[1...index], epsilon)
//         recResults2[] = DouglasPeucker(PointList[index...end], epsilon)

//         # Build the result list
//         ResultList[] = {recResults1[1...length(recResults1) - 1], recResults2[1...length(recResults2)]}
//     } else {
//         ResultList[] = {PointList[1], PointList[end]}
//     }
//     # Return the result
//     return ResultList[]