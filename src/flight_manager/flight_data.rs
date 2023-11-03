
const IGC_DATE: &str = "HFDTE";
#[derive(Debug)]
pub struct FlightData{
    pub date: String,
}

impl FlightData {
    pub fn load(igc: &String) -> Self
    {
        let flight_calculator: FlightData = FlightData{
            date: FlightData::extract_date(&igc),
        };

        return flight_calculator;
    }

    fn extract_date(igc: &String) -> String
    {
        let mut date_start_index: usize = igc.find(IGC_DATE).unwrap() + IGC_DATE.len();
        let date_end_index: usize = igc[date_start_index..].find("\n").unwrap() + date_start_index;

        let date: &str = igc.get(date_start_index..date_end_index).unwrap();

        //Search the first numeric value on the line if the file doesn't respect the format
        let mut c: std::str::Chars<'_> = date.chars();
        while !c.next().unwrap().is_numeric() {
            date_start_index += 1;
        }

        let date: &str = igc.get(date_start_index..date_start_index + 6).unwrap();

        return format!("20{}-{}-{}",date.get(4..6).unwrap(),date.get(2..4).unwrap(),date.get(0..2).unwrap());
    }
}