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
const D_TARGET:f32 = D_STOP - 0.05;
const W_MAX:f32 = 0.8;

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

/// 前方180度のうち、45度刻みで最も前が開いている方向を選び転換する
/// 進行方向の障害物までの距離が、D_SLOWより大きければVMAX,それ以下なら段階的に減速し、D_STOP以下は回転する
pub fn expert(beams: &[f32; NUM_BEAMS]) -> Action {
    let target_beams = [beams[NUM_BEAMS-6],beams[NUM_BEAMS-5],beams[NUM_BEAMS-4],beams[NUM_BEAMS-3],beams[NUM_BEAMS-2],beams[NUM_BEAMS-1], beams[0],beams[1],beams[2], beams[3], beams[4], beams[5], beams[6]];
    let index_array = [6,7,5,8,4,9,3,10,2,11,1];
    let mut max_value = 0f32;
    let mut max_idx:usize = 6;
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
        let mut min_value = f32::INFINITY;
        for j in 0..3{
            if min_value > target_beams[i-1+j]{
                min_value = target_beams[i-1+j];
            }
        }
        if max_value == min_value {
            max_idx = *i;
            break;
        }
    }
    let mut angle = - PI / 2.0 + max_idx as f32 * PI /12.0;
    if angle < -W_MAX {
        angle = -W_MAX;
    } else if angle > W_MAX {
        angle = W_MAX;
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
    } else if front_min < D_STOP{
        Action { linear: 0.0, angular: W_MAX }
    } else {
        Action { linear: VMAX * (front_min - D_TARGET) /(D_SLOW - D_TARGET), angular: angle }
    }
    
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_when_blocked() {
        let mut b = [3.0; NUM_BEAMS];
        b[0] = 0.3;
        assert_eq!(expert(&b).linear, VMAX*(0.3-D_TARGET)/(D_SLOW - D_TARGET));
        assert_eq!(expert(&b).angular, PI/6.0);
    }

    //塞がっているときにargmaxの向きへ回ると、argmaxの向きと固定の旋回の向きが食い違って往復する。
    //そのため、回転を一方向に固定する
    #[test]
    fn test_turn_left_when_blocked() {
        let mut beams1 = [0.0f32 ;NUM_BEAMS];
        beams1[1] = 3.0;
        beams1[2] = 3.0;
        beams1[3] = 3.0;
        assert_eq!(expert(&beams1).linear, 0.0);
        assert_eq!(expert(&beams1).angular, W_MAX);

        let mut beams2 = [0.0f32 ;NUM_BEAMS];
        beams2[NUM_BEAMS-1] = 3.0;
        beams2[NUM_BEAMS-2] = 3.0;
        beams2[NUM_BEAMS-3] = 3.0;
        assert_eq!(expert(&beams2).linear, 0.0);
        assert_eq!(expert(&beams2).angular, W_MAX);
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
        assert_eq!(expert(&neg_b).angular, W_MAX);
        assert_eq!(expert(&nan_b).linear, 0.0);
        assert_eq!(expert(&nan_b).angular, W_MAX);      
        assert_eq!(expert(&b_mix).linear, 0.0);
        assert_eq!(expert(&b_mix).angular, W_MAX);        
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
