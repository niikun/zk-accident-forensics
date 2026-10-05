use std::env;
use std::fs;
use std::path::Path;
use csv;
use  policy::{preprocess_beams, expert, Action};

#[derive(Debug)]
pub struct CmdVel {
    pub time: u64,
    pub linear_x: f64,
    pub angular_z: f64,
}
#[derive(Debug)]
pub struct Scan {
    pub time_stamp: u64,
    pub log_time: u64,
    pub ranges: Vec<f32>
}

pub fn main() {
    let args:Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_scan_data>", args[0]);
        std::process::exit(1);
    }
    let path = &args[1];
    let file_cmd_vel = Path::new(path).join("data_cmd_vel.csv");
    let file_scan = Path::new(path).join("data_scan.csv");
    let mut cmd_vels: Vec<CmdVel> = Vec::new();
    let mut scans: Vec<Scan> = Vec::new();
    let mut rd_cmd_vel = csv::Reader::from_reader(fs::File::open(&file_cmd_vel).unwrap());
    for result in rd_cmd_vel.records() {
        let record = result.unwrap();
        let time:u64 = record[0].parse().unwrap();
        let lin_x:f64 = record[1].parse().unwrap();
        let ang_z:f64 = record[6].parse().unwrap();
        cmd_vels.push(CmdVel{time:time, 
                linear_x:lin_x, 
                angular_z:ang_z});
    }

    let mut rd_scan = csv::Reader::from_reader(fs::File::open(&file_scan).unwrap());
    for result in rd_scan.records() {
        let record = result.unwrap();
        let time_stamp:u64 = record[0].parse().unwrap();
        let log_time: u64 = record[1].parse().unwrap();
        let ranges: Vec<f32> = record.iter()
                .skip(2)
                .map(|r| r.parse::<f32>().unwrap())
                .collect();  
        scans.push(Scan{time_stamp:time_stamp, 
            log_time:log_time, 
            ranges:ranges});
    }
    
    println!("scan {:?}", scans.last().unwrap());
}