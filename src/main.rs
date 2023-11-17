use clap::{arg, command};
use logbook::FlightPoint;
use std::{fs, path};
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

        let path = Path::new(s);
        Logbook::load_and_store(path).unwrap();
    }

    if args.get_flag("history") {
        let flights = FlightTable::select_all().unwrap();
        println!("{} flights found",flights.len());

        for flight in flights
        {
            println!("{} - {} - {:3}min - {:3}km - {:3}pts - {}",flight.flight_id,flight.date,flight.duration,flight.distance/1000,flight.score/1000, flight.code);
        }
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
    use crate::logbook::{Logbook, flight_table::FlightTable, site_table::SiteTable, wing_table::WingTable, tag_table::TagTable};

    #[test]
    fn add_flight_site_wing_tag() {

        Logbook::create().unwrap();
        let (mut flight,scorer) = Logbook::load(Path::new("./test.igc")).unwrap();

        SiteTable::update(format!("name='bisanne'"), format!("site_id={}",flight.takeoff_id)).unwrap();

        let wing = WingTable { wing_id: 0, name: "Supair Savage".to_string(), info: "Very cool".to_string(), def: true };

        WingTable::store(wing).unwrap();

        flight.wing_id = WingTable::get_default_wing().unwrap().wing_id;

        let (track,score,code) = Logbook::get_score(scorer).unwrap();

        flight.track = Some(track);
        flight.score = score;
        flight.code = code;

        FlightTable::store(flight).unwrap();

        let (mut flight,scorer) = Logbook::load(Path::new("./test1.igc")).unwrap();

        let (track,score,code) = Logbook::get_score(scorer).unwrap();

        flight.track = Some(track);
        flight.score = score;
        flight.code = code;

        FlightTable::store(flight).unwrap();

        let tag = TagTable{
            tag_id:0,
            name: "Cross".to_string(),
        };
        TagTable::store(tag).unwrap();

        TagTable::associate(2, 1).unwrap();

        let tags = TagTable::search("cr".to_string()).unwrap();

        let flights = FlightTable::get_by_tag(tags).unwrap();
        assert_eq!(flights.len(), 1);
        assert_eq!(flights[0].flight_id, 2);

        let sites = SiteTable::search("an".to_string()).unwrap();

        let flights = FlightTable::get_by_site(sites).unwrap();
        assert_eq!(flights.len(), 1);
        assert_eq!(flights[0].flight_id, 1);
    }
}

fn _to_file(trace: &Vec<FlightPoint>) {
    let path = "./trace";

    let mut output = fs::File::create(path).unwrap();

    for pt in trace {
        writeln!(output, "{},{}", pt.lat, pt.long).unwrap();
    }
}