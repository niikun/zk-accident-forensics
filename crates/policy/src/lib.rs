//! LiDAR観測 → 速度指令の方策。ロボットのノードとSP1 guestで同一コードを共有する。
#![no_std]

/// 方策に入力するLiDARビーム数（/scanをダウンサンプルした値）
pub const NUM_BEAMS: usize = 24;
const NUM_RANGES: usize = 360 / NUM_BEAMS;

/// 速度指令（固定小数点化は学習後に置き換える）
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub linear: f32,
    pub angular: f32,
}
pub fn preprocess_beams(beams_raw : &[f32], range_max: f32) -> Result<[f32; NUM_BEAMS], &'static str> {
    if beams_raw.len() != NUM_BEAMS * NUM_RANGES {
        return Err("beams_raw length is not NUM_BEAMS * NUM_RANGES");
    }
    let mut beams_raw_shaped = [0.0; NUM_BEAMS * NUM_RANGES];
    for i in 0..beams_raw.len() {
        beams_raw_shaped[i] = beams_raw[(i+353)%(beams_raw.len())];
    }
    let mut beams = [0.0; NUM_BEAMS];
    for i in 0..NUM_BEAMS{
        let targets:&mut [f32] = &mut beams_raw_shaped[i * NUM_RANGES..i * NUM_RANGES + NUM_RANGES];
        let mut min_score = f32::INFINITY;
        for target in targets.iter_mut(){
            let b:f32 = if *target == f32::INFINITY{ 
                range_max
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
    let front_beams = [beams[0],beams[1],beams[NUM_BEAMS-1]];
    let front = front_beams[0].min(front_beams[1]).min(front_beams[2]);
    
    if front < 0.5 {
        Action { linear: 0.0, angular: 0.8 }
    } else {
        Action { linear: 0.2, angular: 0.0 }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_when_blocked() {
        let mut b = [3.0; NUM_BEAMS*NUM_RANGES];
        b[0] = 0.3;
        let b2 = preprocess_beams(&b,10.0).unwrap();
        assert_eq!(expert(&b2).linear, 0.0);
        assert_eq!(expert(&b2).angular, 0.8);
    }

    #[test]
    fn test_inf_case() {
        let b = preprocess_beams(&[f32::INFINITY; NUM_BEAMS*NUM_RANGES],10.0).unwrap();
        let neg_b = preprocess_beams(&[f32::NEG_INFINITY; NUM_BEAMS*NUM_RANGES],10.0).unwrap();
        let nan_b = preprocess_beams(&[f32::NAN; NUM_BEAMS*NUM_RANGES],10.0).unwrap();
        let mut mixed = [3.0_f32; NUM_BEAMS*NUM_RANGES];
        mixed[0] = f32::NAN;   // 正面だけ計測失敗
        let b_mix = preprocess_beams(&mixed,10.0).unwrap();
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
        let mut beams_raw = [100.0; NUM_BEAMS * NUM_RANGES];
        beams_raw[355] = 0.5;
        beams_raw[10] = f32::INFINITY;
        beams_raw[25] = f32::NEG_INFINITY;
        beams_raw[40] = f32::NAN;
        let beams = preprocess_beams(&beams_raw, 10.0).unwrap();
        assert_eq!(beams[0],0.5);
        assert_eq!(beams[1],10.0);
        assert_eq!(beams[2],0.0);
        assert_eq!(beams[3],0.0);    
    }
}
