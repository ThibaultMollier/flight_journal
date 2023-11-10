use clap::{arg, command};
use flight_manager::flight_data::Statistic;
use flight_manager::FlightManager;
use std::path::Path;
use std::time::Instant;

pub mod flight_manager;

fn add(fm: &FlightManager, path: &String) {
    let flights = fm.load_traces(Path::new(path));
    let fl = flights.clone();
    fm.store_flights(flights);
    for f in fl {
        println!("{}", f.unwrap().date);
    }
}

fn history(fm: &FlightManager) {
    let flights = fm.flights_history(Some(2023), None);

    for flight in flights {
        println!("{}\t - \t{}\t - \t{}h{}min\t - \t{:3.1}km", flight.id.unwrap(), flight.date, flight.duration / 60, flight.duration % 60, flight.distance as f32/1000.0);
    }

    // dbg!(flights.statistic());
}

fn select(fm: &FlightManager, id: u32) {
    let flight = fm.get_flights_by_id(id);

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
     
    let flightmanager: FlightManager = FlightManager::new().unwrap();

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