use flight_manager::FlightManager;
use std::{fs, fs::metadata, path::Path};
use clap::{arg, command};

mod flight_manager;

fn search_igc<F>(path: &String, f: &F) where
    F: Fn(String)
{
    if metadata(path).unwrap().is_dir()
    {
        let dir: fs::ReadDir = match fs::read_dir(path){
            Ok(s) => s,
            Err(_) => return,
        };
        for subdir in dir
        {
            search_igc(&subdir.unwrap().path().to_string_lossy().to_string(),f);
        }
    }else if metadata(path).unwrap().is_file() {
        if Path::new(path).extension().map(|s| s == "igc").unwrap_or(false)
        {
            f(path.to_string());
        }
    }
}

fn add(fm: &FlightManager,path: &String)
{
    search_igc(path,&|file_path| {
        println!("{}",file_path);
        fm.store(&file_path);
    });
}

fn history(fm: &FlightManager)
{
    let flights: Vec<flight_manager::Flight> = fm.history();

    for flight in flights {
        println!("{} - {} - {}min", flight.id, flight.data.date, flight.data.duration);
    }
}

fn select(fm: &FlightManager, id:u32)
{
    let flight: flight_manager::Flight = fm.get(id);

    println!("{:?}",flight);
}

fn main() {
    let args: clap::ArgMatches = command!()
        .arg(arg!(--add <PATH>).required(false))
        .arg(arg!(--history).required(false))
        .arg(arg!(--select <ID>).required(false))
        .get_matches();

    let flightmanager: FlightManager = FlightManager::new();

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
}
