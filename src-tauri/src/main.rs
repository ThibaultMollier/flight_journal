// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use logbook::flight_table::FlightTable;

mod logbook;
mod flight_track;

#[tauri::command]
fn history() -> Result<Vec<FlightTable>, String>{
    let flights = FlightTable::select_all().map_err(|err| err.to_string())?;
    Ok(flights)
}

#[tauri::command]
fn select(id: u32) -> Result<FlightTable,String>{
    let flight = FlightTable::get(id).map_err(|err| err.to_string())?;
    Ok(flight)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![history,select])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
