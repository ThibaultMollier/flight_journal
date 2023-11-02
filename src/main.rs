use flight_manager::FlightManager;

mod flight_manager;

fn main() {
    let flightmanager = FlightManager::new();
    flightmanager.store("./");
}
