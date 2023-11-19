use chrono::NaiveDate;
use crate::logbook::FlightPoint;
use anyhow::{Result, bail};

const IGC_HEADER: &str = "H";
const IGC_DATE: &str = "FDTE";

const IGC_RECORD: &str = "B";
const IGC_CHECK: &str = "G";

pub struct IgcReader{
    pub date        :NaiveDate,
    pub track       :Vec<FlightPoint>,
    pub check       :String,
}

impl IgcReader {
    pub fn read(raw_igc: &String) -> Result<Self>
    {
        let mut track: Vec<FlightPoint> = Vec::new();
        let mut date: NaiveDate = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let igc_lines = raw_igc.lines();
        let mut check = "".to_string();

        for line in igc_lines {
            if line.chars().nth(0) == IGC_RECORD.chars().nth(0) {
                track.push(Self::process_record(line, &date));
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
        
        Ok(IgcReader { 
                    date, 
                    track,
                    check 
                })
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

    fn to_decimal(degree: u32, minute: f32, snew: char) -> f32 {
        let mut decimal: f32 = minute / 60.0;
        decimal += degree as f32;

        if snew == 'S' || snew == 'W' {
            decimal = -decimal;
        }

        decimal
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

        let time = date.and_hms_opt(hour, min, sec).unwrap_or_default();

        FlightPoint {
            time,
            long,
            lat,
            alt,
            alt_gps,
        }
    }
}