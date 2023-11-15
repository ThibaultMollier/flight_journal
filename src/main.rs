use clap::{arg, command};
use flight_manager::flight_data::Statistic;
use flight_manager::FlightManager;
use flight_manager::flight_data::trace_manager::FlightPoint;
use geoutils::Location;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

pub mod flight_manager;

fn add(fm: &FlightManager, path: &String) {
    let flights = fm.load_traces(Path::new(path));
    println!("{} flights found",flights.len());
    fm.store_flights(flights).unwrap();
}

fn history(fm: &FlightManager) {
    let flights = fm.flights_history(None, None).unwrap();

    for flight in flights {
        println!("{}\t - \t{}\t - \t{}h{}min\t - \t{:3.1}km \t{} => {}", flight.id.unwrap(), flight.date, flight.duration / 60, flight.duration % 60, flight.distance as f32/1000.0,flight.takeoff.unwrap_or(0),flight.landing.unwrap_or(0));
    }

    // dbg!(flights.statistic());
}

fn select(fm: &FlightManager, id: u32) {
    let flight = fm.get_flights_by_id(id).unwrap();

    println!("{:?}", flight);
}

fn delete(fm: &FlightManager, id: u32) {
    fm.delete_flight(id).unwrap();
}

fn main() {
    let args: clap::ArgMatches = command!()
        .arg(arg!(--add <PATH>).required(false))
        .arg(arg!(--history).required(false))
        .arg(arg!(--select <ID>).required(false))
        .arg(arg!(--delete <ID>).required(false))
        .get_matches();

    let now = Instant::now();

    // let loc = Location::new(45.746433357719404, 6.50472513037558);
    // let d = loc.distance_to(&Location::new(45.746433357719404 , 6.50472513037558 + 0.005)).unwrap().meters();

    // dbg!(d);
     
    let flightmanager: FlightManager = FlightManager::new().unwrap();

    // let _f = flightmanager.get_flights_by_sites("bis".to_string());

    // println!("{}",_f.len());

    if let Some(s) = args.get_one::<String>("add") {
        add(&flightmanager, s)
    }

    if args.get_flag("history") {
        history(&flightmanager)
    }

    if let Some(i) = args.get_one::<String>("select") {
        select(&flightmanager, i.parse().unwrap())
    }

    if let Some(i) = args.get_one::<String>("delete") {
        delete(&flightmanager, i.parse().unwrap())
    }

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::{flight_manager::{FlightManager, flight_data::{Wing, Tag, Statistic, trace_manager::FlightTrace}}, _to_file};

    #[test]
    fn add_flight_site_wing_tag() {
        let flightmanager: FlightManager = FlightManager::new().unwrap();
        let mut flights = flightmanager.load_traces(Path::new("./test.igc"));
        assert_eq!(flights.len(), 1);

        flightmanager.edit_site(flights[0].takeoff.unwrap(), Some("Bisanne".to_string()), None, None, None, Some("Best place".to_string())).unwrap();

        let wing = Wing{
            id: 0,
            name: "Supair Savage".to_string(),
            info: "".to_string(),
            default: Some(true),
        };
        flightmanager.store_wing(wing).unwrap();

        let wing = flightmanager.get_wing(None, Some("Supair Savage".to_string())).unwrap();
        assert_eq!(wing.id, 1);
        flights[0].wing = wing.id;

        flightmanager.store_flights(flights).unwrap();

        let flights = flightmanager.load_traces(Path::new("./test1.igc"));
        assert_eq!(flights.len(), 1);
        assert_ne!(flights[0].trace.clone().unwrap().raw_igc.len(),0);
        flightmanager.store_flights(flights).unwrap();

        let tag = Tag
        {
            id:0,
            name: "Cross".to_string(),
        };
        flightmanager.store_tag(tag).unwrap();

        flightmanager.associate_tag(1, 2).unwrap();

        let flights = flightmanager.get_flights_by_tags("cr".to_string()).unwrap();
        assert_eq!(flights.len(), 1);
        assert_eq!(flights[0].id, Some(2));

        let flights = flightmanager.get_flights_by_sites("an".to_string()).unwrap();
        assert_eq!(flights.len(), 1);
        assert_eq!(flights[0].id, Some(1));

        let flights = flightmanager.flights_history(None, None).unwrap();
        dbg!(flights.statistic());

        let _f = flightmanager.get_flights_by_id(2).unwrap();
        // let points = FlightTrace::triangle(&f.trace.unwrap().simplified_trace);
        // _to_file(&points.0);
    }
}

fn _to_file(trace: &Vec<FlightPoint>) {
    let path = "./trace";

    let mut output = fs::File::create(path).unwrap();

    for pt in trace {
        writeln!(output, "{},{}", pt.lat, pt.long).unwrap();
    }
}