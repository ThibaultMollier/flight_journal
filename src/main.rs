use flight_manager::FlightManager;
use std::{env, fs, fs::metadata, path::Path};

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("not enough arguments");
    }

    let flightmanager: FlightManager = FlightManager::new();

    let path: &String = &args[1];

    search_igc(path,&|file_path| {
        flightmanager.store(&file_path);
    });
    
    let flights: Vec<flight_manager::Flight> = flightmanager.history();

    for flight in flights {
        println!("{}",flight.data.date);
    }
}
