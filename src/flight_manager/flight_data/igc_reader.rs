use crate::flight_manager::flight_data::FlightPoint;
use chrono::{Utc, TimeZone, Datelike};

const IGC_HEADER: &str = "H";
const IGC_DATE: &str = "FDTE";

const IGC_RECORD: &str = "B";

pub struct IgcReader{
    pub trace: Vec<FlightPoint>,
    pub date: chrono::DateTime<Utc>,
}

impl IgcReader {
    
    pub fn read(igc: &String) -> Self
    {
        let mut trace: Vec<FlightPoint> = Vec::new();
        let mut date: chrono::DateTime<Utc> = Utc.with_ymd_and_hms(0, 1, 1, 0, 0, 0).unwrap();
        let igc_lines = igc.lines();

        for line in igc_lines
        {
            if line.chars().nth(0) == IGC_RECORD.chars().nth(0)
            {
                trace.push(IgcReader::process_record(line, &date));
            }else if line.chars().nth(0) == IGC_HEADER.chars().nth(0)
            {
                if line[IGC_HEADER.len()..IGC_HEADER.len()+IGC_DATE.len()] == *IGC_DATE
                {
                    date = IgcReader::process_date(line);
                }
            }
        }

        return IgcReader{
            trace,
            date,
        };
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
    
            IgcReader::to_decimal(degree, minute, snew)
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

            IgcReader::to_decimal(degree, minute, snew)
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
}