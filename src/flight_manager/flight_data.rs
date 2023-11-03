use chrono::{Utc, TimeZone, Datelike};

const IGC_HEADER: &str = "H";
const IGC_DATE: &str = "FDTE";

const IGC_RECORD: &str = "B";

#[derive(Debug)]
pub struct FlightData{
    pub date: String,
    pub duration: i64,
}

struct Point{
    time: chrono::DateTime<Utc>,
}

impl FlightData {
    pub fn compute(igc: &String) -> Self
    {
        let igc_lines = igc.lines();
        let mut trace: Vec<Point> = Vec::new();
        let mut date: chrono::DateTime<Utc> = Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0).unwrap();

        for line in igc_lines
        {
            if line.chars().nth(0) == IGC_RECORD.chars().nth(0)
            {
                trace.push(FlightData::process_record(line));
            }else if line.chars().nth(0) == IGC_HEADER.chars().nth(0)
            {
                if line[IGC_HEADER.len()..IGC_HEADER.len()+IGC_DATE.len()] == *IGC_DATE
                {
                    date = FlightData::process_date(line);
                }
            }
        }

        let duration = trace.last().unwrap().time - trace.get(0).unwrap().time;

        let flightdata: FlightData = FlightData {
            date: format!("{}-{:02}-{:02}",date.year(),date.month(),date.day()),
            duration: duration.num_minutes(),
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

    fn process_record(line: &str) -> Point
    {  
        let mut c = line[IGC_RECORD.len()..].chars();
        let hour: u32   = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
        let min: u32    = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();
        let sec: u32    = c.next().unwrap().to_digit(10).unwrap()*10 + c.next().unwrap().to_digit(10).unwrap();

        Point { 
            time: Utc.with_ymd_and_hms(0, 1, 1, hour, min, sec).unwrap() 
        }
    }
}