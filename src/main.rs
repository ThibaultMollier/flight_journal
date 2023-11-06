use flight_manager::FlightManager;
use std::path::Path;
use clap::{arg, command};

mod flight_manager;

fn add(fm: &FlightManager,path: &String)
{
    let flights = fm.load(Path::new(path));
    fm.store(flights);
}

fn history(fm: &FlightManager)
{
    // let flights: Vec<flight_manager::Flight> = fm.history();

    // for flight in flights {
    //     println!("{}\t - \t{}\t - \t{}h{}min\t - \t{:3.1}km", flight.id, flight.data.date, flight.data.duration / 60, flight.data.duration % 60, flight.data.distance as f32/1000.0);
    // }
}

fn select(fm: &FlightManager, id:u32)
{
    // let flight: flight_manager::Flight = fm.get(id);

    // println!("{:?}",flight);
}

fn delete(fm: &FlightManager, id:u32)
{
    // fm.delete(id);
}

fn main() {
    let args: clap::ArgMatches = command!()
        .arg(arg!(--add <PATH>).required(false))
        .arg(arg!(--history).required(false))
        .arg(arg!(--select <ID>).required(false))
        .arg(arg!(--delete <ID>).required(false))
        .get_matches();

    let flightmanager: FlightManager = FlightManager::new().unwrap();

    match args.get_one::<String>("add") {
        Some(s) => add(&flightmanager, s),
        None => (),
    }

    match args.get_flag("history") {
        true => history(&flightmanager),
        false => (),
    }

    match args.get_one::<String>("select") {
        Some(i) => select(&flightmanager, i.parse().unwrap()),
        None => (),
    }

    match args.get_one::<String>("delete") {
        Some(i) => delete(&flightmanager, i.parse().unwrap()),
        None => (),
    }
}
