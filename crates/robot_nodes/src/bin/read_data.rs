use std::env;
use std::fs;
use std::path::Path;
use csv;
use  policy::{preprocess_beams, expert, Action};

pub fn main() {
    let args:Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_scan_data>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];
    let file_cmd_vel = Path::new(path).join("data_cmd_vel.csv");
    let file_scan = Path::new(path).join("data_scan.csv");
    
    let mut rdr = csv::Reader::from_reader(fs::File::open(&file_cmd_vel).unwrap());
    for result in rdr.records() {
        let record = result.unwrap();
        println!("record: {:?}", record);

    // let entries_cmd_vel = fs::read_to_string(file_cmd_vel).unwrap();
    // let entries_scan = fs::read_to_string(file_scan).unwrap();

    // println!("Command velocity entries: {:#?}", entries_cmd_vel);
    // println!("Scan entries: {:#?}", entries_scan);
    }
}