//! LiDAR観測 → 速度指令の方策。ロボットのノードとSP1 guestで同一コードを共有する。
#![no_std]

/// 方策に入力するLiDARビーム数（/scanをダウンサンプルした値）
/// 
use core::f32::consts::PI;
pub const NUM_BEAMS: usize = 24;
const NUM_RANGES_RAW:usize = 360;
const RANGES_PER_BEAM: usize = NUM_RANGES_RAW / NUM_BEAMS;
pub const RANGE_MAX: f32 = 10.0;
const VMAX:f32 = 0.2;
const D_SLOW:f32 = 0.5;
const D_STOP:f32 = 0.25;

/// 速度指令（固定小数点化は学習後に置き換える）
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub linear: f32,
    pub angular: f32,
}

/// 入力の並び（ranges[0]が正面、左回り、1度刻み、360本)
/// RANGE MAXは10.0で設定。front は353度～7度までで設定
/// 置き換えの規則（+inf→RANGE_MAX、-inf→0、NaN→0、区間の最小値）
/// Errになる条件（長さが360でない）
pub fn preprocess_beams(beams_raw : &[f32]) -> Result<[f32; NUM_BEAMS], &'static str> {
    if beams_raw.len() != NUM_RANGES_RAW {
        return Err("beams_raw length is not NUM_RANGES_RAW");
    }
    let mut beams_raw_shaped = [0.0; NUM_RANGES_RAW];
    for i in 0..beams_raw.len() {
        beams_raw_shaped[i] = beams_raw[(i + NUM_RANGES_RAW - RANGES_PER_BEAM / 2)%(beams_raw.len())];
    }
    let mut beams = [0.0; NUM_BEAMS];
    for i in 0..NUM_BEAMS{
        let targets:&[f32] = &beams_raw_shaped[i * RANGES_PER_BEAM..i * RANGES_PER_BEAM + RANGES_PER_BEAM];
        let mut min_score = f32::INFINITY;
        for target in targets.iter(){
            let b:f32 = if *target == f32::INFINITY{ 
                RANGE_MAX
            } else if *target == f32::NEG_INFINITY {
                0.0
            } else if target.is_nan() {
                0.0
            } else {
                *target
            };
            if b < min_score {
                min_score = b;
            }
        }    
        beams[i] = min_score;
    }
    Ok(beams)   
}

/// 仮のルールベース方策（エキスパート）。前方が近ければ旋回、空いていれば直進。
pub fn expert(beams: &[f32; NUM_BEAMS]) -> Action {
    let target_beams = [beams[NUM_BEAMS-6],beams[NUM_BEAMS-5],beams[NUM_BEAMS-4],beams[NUM_BEAMS-3],beams[NUM_BEAMS-2],beams[NUM_BEAMS-1], beams[0],beams[1],beams[2], beams[3], beams[4], beams[5], beams[6]];
    let index_array = [6,7,5,8,4,9,3,10,2,11,1,12,0];
    let mut max_value = 0f32;
    let mut max_idx = 7.0;
    for i in 1..(target_beams.len()-1){
        let mut min_value = f32::INFINITY;
        for j in 0..3{
            if min_value > target_beams[i-1+j]{
                min_value = target_beams[i-1+j];
            }
        }
        if max_value < min_value{
            max_value = min_value;
        }
    }
    for i in index_array.iter(){
        if max_value == target_beams[*i] {
            max_idx = *i as f32;
            break;
        }
    }
    let mut angle = - PI / 2.0 + max_idx * PI /12.0;
    if angle < -0.8 {
        angle = -0.8;
    } else if angle > 0.8 {
        angle = 0.8;
    }
    let front_beams = [target_beams[4], target_beams[5], target_beams[6], target_beams[7], target_beams[8]];
    let mut front_min:f32 = f32::INFINITY;
    for beam in front_beams{
        if front_min > beam {
            front_min = beam;
        }
    }
    if front_min >= D_SLOW{
        Action { linear: VMAX, angular: angle }
    } else if front_min < D_STOP {
        Action { linear: 0.0, angular: angle }
    } else {
        Action { linear: VMAX * (front_min - D_STOP) /(D_SLOW - D_STOP), angular: angle }
    }
    
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_when_blocked() {
        let mut b = [3.0; NUM_BEAMS];
        b[0] = 0.3;
        assert_eq!(expert(&b).linear, 0.0);
        assert_eq!(expert(&b).angular, PI/12.0);
    }

    #[test]
    fn test_inf_case() {
        let b = preprocess_beams(&[f32::INFINITY; NUM_RANGES_RAW]).unwrap();
        let neg_b = preprocess_beams(&[f32::NEG_INFINITY; NUM_RANGES_RAW]).unwrap();
        let nan_b = preprocess_beams(&[f32::NAN; NUM_RANGES_RAW]).unwrap();
        let mut mixed = [3.0_f32; NUM_RANGES_RAW];
        mixed[0] = f32::NAN;   // 正面だけ計測失敗
        let b_mix = preprocess_beams(&mixed).unwrap();
        assert_eq!(expert(&b).linear, 0.2);
        assert_eq!(expert(&b).angular, 0.0);
        assert_eq!(expert(&neg_b).linear, 0.0);
        assert_eq!(expert(&neg_b).angular, 0.8);
        assert_eq!(expert(&nan_b).linear, 0.0);
        assert_eq!(expert(&nan_b).angular, 0.8);      
        assert_eq!(expert(&b_mix).linear, 0.0);
        assert_eq!(expert(&b_mix).angular, 0.8);        
    }

    #[test]
    fn test_preprocess_beams() {
        let mut beams_raw = [100.0; NUM_RANGES_RAW];
        beams_raw[355] = 0.5;
        beams_raw[10] = f32::INFINITY;
        beams_raw[25] = f32::NEG_INFINITY;
        beams_raw[40] = f32::NAN;
        let beams = preprocess_beams(&beams_raw).unwrap();
        assert_eq!(beams[0],0.5);
        assert_eq!(beams[1],10.0);
        assert_eq!(beams[2],0.0);
        assert_eq!(beams[3],0.0);    
    }

    #[test]
    fn test_preprocess_beams_boundary() {
        let beams_raw = [f32::INFINITY; NUM_RANGES_RAW];
        let mut beams_raw1 = beams_raw.clone();
        beams_raw1[352] = 0.5;
        let mut beams_raw2 = beams_raw.clone();
        beams_raw2[353] = 0.5;
        let mut beams_raw3 = beams_raw.clone();
        beams_raw3[359] = 0.5;
        let mut beams_raw4 = beams_raw.clone();
        beams_raw4[0] = 0.5;
        let mut beams_raw5 = beams_raw.clone();
        beams_raw5[7] = 0.5;
        let mut beams_raw6 = beams_raw.clone();
        beams_raw6[8] = 0.5;

        let beams1 = preprocess_beams(&beams_raw1).unwrap();
        let beams2 = preprocess_beams(&beams_raw2).unwrap();
        let beams3 = preprocess_beams(&beams_raw3).unwrap();
        let beams4 = preprocess_beams(&beams_raw4).unwrap();
        let beams5 = preprocess_beams(&beams_raw5).unwrap();
        let beams6 = preprocess_beams(&beams_raw6).unwrap();
        assert_eq!(beams1[23],0.5);
        assert_eq!(beams1[0], 10.0);
        assert_eq!(beams1[1], 10.0);
        assert_eq!(beams2[23],10.0);
        assert_eq!(beams2[0], 0.5);
        assert_eq!(beams2[1], 10.0);
        assert_eq!(beams3[23],10.0);
        assert_eq!(beams3[0], 0.5);
        assert_eq!(beams3[1], 10.0);
        assert_eq!(beams4[23],10.0);
        assert_eq!(beams4[0], 0.5);
        assert_eq!(beams4[1], 10.0);
        assert_eq!(beams5[23],10.0);
        assert_eq!(beams5[0], 0.5);
        assert_eq!(beams5[1], 10.0);
        assert_eq!(beams6[23],10.0);
        assert_eq!(beams6[0], 10.0);
        assert_eq!(beams6[1], 0.5);
        }


}
