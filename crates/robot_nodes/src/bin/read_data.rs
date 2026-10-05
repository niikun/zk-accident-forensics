use csv;
use policy::{expert, preprocess_beams};
use std::env;
use std::fs;
use std::path::Path;

const MAX_GAP_NS: u64 = 10_000_000; // scan-> cmd_vel gap: 10ms

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
    pub ranges: Vec<f32>,
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
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
        let time: u64 = record[0].parse().unwrap();
        let lin_x: f64 = record[1].parse().unwrap();
        let ang_z: f64 = record[6].parse().unwrap();
        cmd_vels.push(CmdVel {
            time: time,
            linear_x: lin_x,
            angular_z: ang_z,
        });
    }

    let mut rd_scan = csv::Reader::from_reader(fs::File::open(&file_scan).unwrap());
    for result in rd_scan.records() {
        let record = result.unwrap();
        let time_stamp: u64 = record[0].parse().unwrap();
        let log_time: u64 = record[1].parse().unwrap();
        let ranges: Vec<f32> = record
            .iter()
            .skip(2)
            .map(|r| r.parse::<f32>().unwrap())
            .collect();
        scans.push(Scan {
            time_stamp: time_stamp,
            log_time: log_time,
            ranges: ranges,
        });
    }
    let mut matched_count = 0;
    let mut no_action_count = 0;
    let mut unmatched_count = 0;
    for scan in scans.iter() {
        if let Some(cmd) = cmd_vels
            .iter()
            .find(|c| c.time > scan.log_time && c.time - scan.log_time < MAX_GAP_NS)
        {
            let action = expert(&preprocess_beams(&scan.ranges).unwrap());
            if action.linear as f64 != cmd.linear_x || action.angular as f64 != cmd.angular_z {
                unmatched_count += 1;
            } else {
                matched_count += 1;
            }
        } else {
            no_action_count += 1;
        }
    }
    println!("matched_count: {:?}", matched_count);
    println!("no_action_count: {:?}", no_action_count);
    println!("unmatched_count: {:?}", unmatched_count);
}
