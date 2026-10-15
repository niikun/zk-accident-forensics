use std::{cmp::max, fmt::Error};

use policy::{Action, SpeedMode, angular_to_class, class_to_angular, forward, normalize_beam, speed_mode_to_linear};

#[test]    
fn read_csv_and_forward(){
    let path = "../../datas/test.csv";
    let mut reader = csv::Reader::from_path(&path)
        .unwrap_or_else(|e| panic!("file can not open :{}", e));
    let mut runs: Vec<String> = Vec::new();
    let mut beams:Vec<Vec<f32>> = Vec::new();
    let mut classes:Vec<usize> = Vec::new();
    let mut modes: Vec<usize> = Vec::new();
    let mut linears: Vec<f32> = Vec::new();
    let mut actions:Vec<Action> = Vec::new();
    for record in reader.records(){
        let mut beam:Vec<f32> = Vec::new();
        for (i, r) in record.unwrap().iter().enumerate() {
            if i == 0 {
                runs.push(r.to_string());
            } else if i == 25usize {
                classes.push(r.parse::<usize>().unwrap());
            } else if i == 26 {
                modes.push(r.parse::<usize>().unwrap());
            } else if i == 27 {
                linears.push(r.parse::<f32>().unwrap());
            } else {
                beam.push(normalize_beam(&r.parse::<f32>().unwrap()));
            }
        }
        beams.push(beam);
    }
    let mut class_match = 0;
    let mut mode_match = 0;
    let mut linear_match = 0;
    for i in 0..beams.len(){
        let (out1, out2, out3) = forward(&beams[i]);   
        let mut max_class_num = f32::NEG_INFINITY;
        let mut class_pred:usize = 0;
        for (m, o1) in out1.iter().enumerate(){
            if *o1 > max_class_num {
                class_pred = m;
                max_class_num = *o1;
            }
        }
        let angular_pred = class_to_angular(class_pred).expect("can not convert class to angular");
        let mut max_mode_num = f32::NEG_INFINITY;
        let mut mode_pred:usize = 0;
        for (n, o2) in out2.iter().enumerate(){
            if *o2 >= max_mode_num{
                mode_pred = n;
                max_mode_num = *o2;
            }
        }
        let speed_mode:SpeedMode = SpeedMode::from_mode(mode_pred).expect("can not convert mode to speedmode");
        let linear_pred = speed_mode_to_linear(speed_mode, out3[0]);
        if class_pred == classes[i]{
            class_match += 1;
        }

        if mode_pred == modes[i]{
            mode_match += 1;
        }
        let action = Action{
            linear:linear_pred,
            angular:angular_pred
        };
        actions.push(action);

    }
    assert_eq!(class_match, 245);
    assert_eq!(mode_match, 307);
    println!("{:?}",actions);
}



