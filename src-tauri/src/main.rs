// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use logbook::flight_table::FlightTable;

mod logbook;
mod flight_track;

#[tauri::command]
fn history() -> Vec<FlightTable>{
    let flights = FlightTable::select_all().unwrap();
    flights
}

#[tauri::command]
fn select(id: u32) -> FlightTable{
    let flight = FlightTable::get(id).unwrap();
    flight
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![history,select])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
