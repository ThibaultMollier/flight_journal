use clap::{arg, command};
use logbook::FlightPoint;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::logbook::Logbook;
use crate::logbook::flight_table::FlightTable;

pub mod logbook;
pub mod flight_track;

fn main() {
    let args: clap::ArgMatches = command!()
        .arg(arg!(--add <PATH>).required(false))
        .arg(arg!(--history).required(false))
        .arg(arg!(--select <ID>).required(false))
        .arg(arg!(--delete <ID>).required(false))
        .get_matches();

    let now = Instant::now();

    Logbook::create().unwrap();

    if let Some(s) = args.get_one::<String>("add") {
        let flights = Logbook::load(Path::new(s)).unwrap();
        println!("{} flights found",flights.len());
        for flight in flights
        {
            FlightTable::store(flight).unwrap();
        }
    }

    if args.get_flag("history") {
        let flights = FlightTable::select_all().unwrap();
        println!("{} flights found",flights.len());
        dbg!(flights);
    }

    if let Some(i) = args.get_one::<String>("select") {
        let flight = FlightTable::select(format!("flight_id={}",i)).unwrap();
        println!("{} {}",flight[0].flight_id,flight[0].date);
    }

    if let Some(i) = args.get_one::<String>("delete") {
        FlightTable::delete(format!("flight_id={}",i)).unwrap()
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn add_flight_site_wing_tag() {
        // let flightmanager: FlightManager = FlightManager::new().unwrap();
        // let mut flights = flightmanager.load_traces(Path::new("./test.igc"));
        // assert_eq!(flights.len(), 1);

        // flightmanager.edit_site(flights[0].takeoff.unwrap(), Some("Bisanne".to_string()), None, None, None, Some("Best place".to_string())).unwrap();

        // let wing = Wing{
        //     id: 0,
        //     name: "Supair Savage".to_string(),
        //     info: "".to_string(),
        //     default: Some(true),
        // };
        // flightmanager.store_wing(wing).unwrap();

        // let wing = flightmanager.get_wing(None, Some("Supair Savage".to_string())).unwrap();
        // assert_eq!(wing.id, 1);
        // flights[0].wing = wing.id;

        // flightmanager.store_flights(flights).unwrap();

        // let flights = flightmanager.load_traces(Path::new("./test1.igc"));
        // assert_eq!(flights.len(), 1);
        // assert_ne!(flights[0].trace.clone().unwrap().raw_igc.len(),0);
        // flightmanager.store_flights(flights).unwrap();

        // let tag = Tag
        // {
        //     id:0,
        //     name: "Cross".to_string(),
        // };
        // flightmanager.store_tag(tag).unwrap();

        // flightmanager.associate_tag(1, 2).unwrap();

        // let flights = flightmanager.get_flights_by_tags("cr".to_string()).unwrap();
        // assert_eq!(flights.len(), 1);
        // assert_eq!(flights[0].id, Some(2));

        // let flights = flightmanager.get_flights_by_sites("an".to_string()).unwrap();
        // assert_eq!(flights.len(), 1);
        // assert_eq!(flights[0].id, Some(1));

        // let flights = flightmanager.flights_history(None, None).unwrap();
        // dbg!(flights.statistic());

        // let _f = flightmanager.get_flights_by_id(2).unwrap();
        // // let points = FlightTrace::triangle(&f.trace.unwrap().simplified_trace);
        // // _to_file(&points.0);
    }
}

fn _to_file(trace: &Vec<FlightPoint>) {
    let path = "./trace";

    let mut output = fs::File::create(path).unwrap();

    for pt in trace {
        writeln!(output, "{},{}", pt.lat, pt.long).unwrap();
    }
}