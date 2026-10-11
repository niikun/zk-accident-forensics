use csv::Writer;
use policy::{angular_to_class, beams_to_mode};
use policy::{expert, preprocess_beams, Action, SpeedMode};
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

const MAX_GAP_NS: u64 = 20_000_000; // scan-> cmd_vel gap: 20ms

#[derive(Debug)]
pub struct CmdVel {
    pub log_time: u64,
    pub linear_x: f64,
    pub angular_z: f64,
}
#[derive(Debug)]
pub struct Scan {
    pub time_stamp: u64,
    pub log_time: u64,
    pub ranges: Vec<f32>,
}

fn main() -> Result<(), Box<dyn Error>> {
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
            log_time: time,
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
            .min_by_key(|c| c.log_time.abs_diff(scan.log_time))
            .filter(|c| c.log_time.abs_diff(scan.log_time) <= MAX_GAP_NS)
        {
            let action = expert(&preprocess_beams(&scan.ranges).unwrap());
            if action.linear as f64 != cmd.linear_x || action.angular as f64 != cmd.angular_z {
                unmatched_count += 1;
                eprintln!("Unmatched:log_time: {}, action_linear: {}, action_angular: {}, cmd_linear: {}, cmd_angular: {}, gap: {} ns", 
                    scan.log_time,
                    action.linear as f64,
                    action.angular as f64,
                    cmd.linear_x,
                    cmd.angular_z,
                    cmd.log_time.abs_diff(scan.log_time)
                );
            } else {
                matched_count += 1;
            }
        } else {
            no_action_count += 1;
        }
    }
    println!(
        "total length: {:?}, matched: {:?}, unmatched: {:?}, no_action: {:?}",
        scans.len(),
        matched_count,
        unmatched_count,
        no_action_count
    );
    if unmatched_count > 0 {
        eprintln!("unmatched count is greater than 0, exiting with error code 1");
        std::process::exit(1);
    }
    let write_path = Path::new(path).join("data.csv");
    let mut wtr = Writer::from_path(write_path)?;
  
    let mut header: Vec<String> = Vec::new();
    for i in 0..24 {
        let beam = format!("b{}", i);
        header.push(beam);
    }
    header.push(String::from("class"));
    header.push(String::from("mode"));
    header.push(String::from("linear"));
    wtr.write_record(header)?;
    for scan in scans {
        let beams = preprocess_beams(&scan.ranges)?;
        let mut result: Vec<String> = Vec::new();
        for beam in beams {
            result.push(beam.to_string());
        }
        let action: Action = expert(&beams);
        let angular = action.angular;
        let linear = action.linear;
        let class = angular_to_class(angular)?;
        let mode: usize = match beams_to_mode(&beams).0 {
            SpeedMode::Max => 0,
            SpeedMode::Slow => 1,
            SpeedMode::Stop => 2,
        };
        result.push(class.to_string());
        result.push(mode.to_string());
        result.push(linear.to_string());
        wtr.write_record(result)?;
    }
    wtr.flush()?;
    Ok(())
}
