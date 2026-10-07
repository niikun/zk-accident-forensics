//! LiDAR観測 → 速度指令の方策。ロボットのノードとSP1 guestで同一コードを共有する。
#![no_std]

use core::{f32::consts::PI, fmt::Error};

/// 方策に入力するLiDARビーム数（/scanをダウンサンプルした値）
pub const NUM_BEAMS: usize = 24;
const NUM_RANGES_RAW: usize = 360;
const RANGES_PER_BEAM: usize = NUM_RANGES_RAW / NUM_BEAMS;
/// RANGEを有限に限定するため、10.0で設定。sdfのLiDAR設定と揃える
pub const RANGE_MAX: f32 = 10.0;
const FRONT_BEAMS_LEN: usize = NUM_BEAMS / 2 + 1;
/// 最大の前進速度(m/s)
pub const VMAX: f32 = 0.2;
/// 衝突を避けるためにこの距離になったら、速度を落とす
const D_SLOW: f32 = 0.5;
/// 衝突を避けるためにこの距離になったら、止まる
const D_STOP: f32 = 0.25;
/// 衝突を避けるためにこの距離をターゲットに速度を調整する
/// ストップより短めに設定しないとD_STOPに到達せず停滞するため設定
const D_TARGET: f32 = D_STOP - 0.05;
/// 角度の上限（rad）と、塞がったときの旋回の角速度（rad/s）
pub const W_MAX: f32 = 0.8;
/// 挙動を安定させるため、距離がほぼ同じ場合、差がこれ未満なら同点とみなす
const TIE_EPS: f32 = 0.01;
/// 回転の9クラス
pub const NUM_ANGULAR_CLASSES: usize = 9;

/// 速度指令（固定小数点化は学習後に置き換える）
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub linear: f32,
    pub angular: f32,
}

/// 入力の並び（ranges[0]が正面、左回り、1度刻み、360本)
/// RANGE_MAXは10.0で設定。front は353度～7度までで設定
/// 置き換えの規則（+inf→RANGE_MAX、-inf→0、NaN→0、区間の最小値）
/// Errになる条件（長さが360でない）
pub fn preprocess_beams(beams_raw: &[f32]) -> Result<[f32; NUM_BEAMS], &'static str> {
    if beams_raw.len() != NUM_RANGES_RAW {
        return Err("beams_raw length is not NUM_RANGES_RAW");
    }
    let mut beams_raw_shaped = [0.0; NUM_RANGES_RAW];
    for i in 0..beams_raw.len() {
        beams_raw_shaped[i] =
            beams_raw[(i + NUM_RANGES_RAW - RANGES_PER_BEAM / 2) % (beams_raw.len())];
    }
    let mut beams = [0.0; NUM_BEAMS];
    for i in 0..NUM_BEAMS {
        let targets: &[f32] =
            &beams_raw_shaped[i * RANGES_PER_BEAM..i * RANGES_PER_BEAM + RANGES_PER_BEAM];
        let mut min_score = f32::INFINITY;
        for target in targets.iter() {
            let b: f32 = if *target == f32::INFINITY {
                RANGE_MAX
            } else if *target == f32::NEG_INFINITY || target.is_nan() {
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

/// 15度刻みの13方向から、3区間（45度）の窓の最小値で選ぶ。同点（TIE_EPS未満の差）は正面に近い方
/// 進行方向の障害物までの距離が、D_SLOWより大きければVMAX,それ以下なら段階的に減速し、D_STOP未満で、その場で左に旋回する
pub fn expert(beams: &[f32; NUM_BEAMS]) -> Action {
    // 前方‐90°~+90°のbeamを集約。angularの計算に使用
    let mut target_beams = [0.0; FRONT_BEAMS_LEN];
    for i in 0..FRONT_BEAMS_LEN {
        let idx = (NUM_BEAMS - FRONT_BEAMS_LEN / 2 + i) % NUM_BEAMS;
        target_beams[i] = beams[idx];
    }
    // 現在前方に近い順番に抽出するために設定。argmaxの際、現在の前方に近い方を選べるように設定
    let index_array = [6, 7, 5, 8, 4, 9, 3, 10, 2, 11, 1];
    let mut max_value = 0.0f32;
    let mut min_values = [f32::INFINITY; FRONT_BEAMS_LEN];
    let mut max_idx: usize = FRONT_BEAMS_LEN / 2;
    for i in 1..(target_beams.len() - 1) {
        let mut min_value = f32::INFINITY;
        // 各位置の左右1つずつ計3つのbeamsのうち最も近い値を抽出
        for j in 0..3 {
            if min_value > target_beams[i - 1 + j] {
                min_value = target_beams[i - 1 + j];
            }
        }
        min_values[i] = min_value;
        if max_value < min_value {
            max_value = min_value;
        }
    }
    for i in index_array.iter() {
        if max_value - TIE_EPS < min_values[*i] {
            max_idx = *i;
            break;
        }
    }
    let angle = -PI / 2.0 + max_idx as f32 * 2.0 * PI / NUM_BEAMS as f32;
    let angle_shaped = angle.clamp(-W_MAX, W_MAX);
    // Linearの速度を調べるために前方5つのbeamsの最小値をもとに計算
    let mut front_min: f32 = f32::INFINITY;
    for i in 4..=8 {
        if front_min > target_beams[i] {
            front_min = target_beams[i];
        }
    }
    // front_min < D_STOP && max_idxが前方(6)の場合も入れていたが、
    // front_min < D_STOPと競合して振動して、止まってしまうため削除
    if front_min >= D_SLOW {
        Action {
            linear: VMAX,
            angular: angle_shaped,
        }
    } else if front_min < D_STOP {
        Action {
            linear: 0.0,
            angular: W_MAX,
        }
    } else {
        Action {
            linear: VMAX * (front_min - D_TARGET) / (D_SLOW - D_TARGET),
            angular: angle_shaped,
        }
    }
}

fn class_to_angular(class: usize) -> Result<f32,Error> {
    if class >= NUM_ANGULAR_CLASSES {
        return Err(Error);
    }
    let angle = -PI / 2.0 + (class+2) as f32 * 2.0 * PI / NUM_BEAMS as f32;
    let angle_shaped = angle.clamp(-W_MAX, W_MAX);
    Ok(angle_shaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    // preprocessを通さず、24本beamsでexpert単体の挙動を確認
    #[test]
    fn turns_when_blocked() {
        let mut b = [3.0; NUM_BEAMS];
        b[0] = 0.3;
        assert_eq!(
            expert(&b).linear,
            VMAX * (0.3 - D_TARGET) / (D_SLOW - D_TARGET)
        );
        assert_eq!(expert(&b).angular, PI / 6.0);
    }

    // ふさがっているときにargmaxの向きへ回ると、argmaxの向きと固定の回転の向きが食い違って振動して止まってしまう
    // そのため、回転を一方向に固定する
    #[test]
    fn test_turn_left_when_blocked() {
        let mut beams1 = [0.0f32; NUM_BEAMS];
        beams1[3] = 3.0;
        beams1[4] = 3.0;
        beams1[5] = 3.0;
        assert_eq!(expert(&beams1).linear, 0.0);
        assert_eq!(expert(&beams1).angular, W_MAX);

        let mut beams2 = [0.0f32; NUM_BEAMS];
        beams2[NUM_BEAMS - 1] = 3.0;
        beams2[NUM_BEAMS - 2] = 3.0;
        beams2[NUM_BEAMS - 3] = 3.0;
        assert_eq!(expert(&beams2).linear, 0.0);
        assert_eq!(expert(&beams2).angular, W_MAX);
    }
    // beams_raw->preprocess_beams->expertの流れを、ifで設定した様々な値で確認
    #[test]
    fn test_inf_case() {
        let b = preprocess_beams(&[f32::INFINITY; NUM_RANGES_RAW]).unwrap();
        let neg_b = preprocess_beams(&[f32::NEG_INFINITY; NUM_RANGES_RAW]).unwrap();
        let nan_b = preprocess_beams(&[f32::NAN; NUM_RANGES_RAW]).unwrap();
        let mut mixed = [3.0_f32; NUM_RANGES_RAW];
        mixed[0] = f32::NAN; // 正面だけ計測失敗
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
    // preprocessの挙動を確認。対象はifで場合分けした値を含む。
    #[test]
    fn test_preprocess_beams() {
        let mut beams_raw = [100.0; NUM_RANGES_RAW];
        beams_raw[355] = 0.5;
        beams_raw[10] = f32::INFINITY;
        beams_raw[25] = f32::NEG_INFINITY;
        beams_raw[40] = f32::NAN;
        let beams = preprocess_beams(&beams_raw).unwrap();
        assert_eq!(beams[0], 0.5);
        assert_eq!(beams[1], 10.0);
        assert_eq!(beams[2], 0.0);
        assert_eq!(beams[3], 0.0);
    }
    // beams_raw(360本)->preprocess->beams(24本)にまとめる際、
    // 境界の値がちゃんと分けられているか確認
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
        assert_eq!(beams1[23], 0.5);
        assert_eq!(beams1[0], 10.0);
        assert_eq!(beams1[1], 10.0);
        assert_eq!(beams2[23], 10.0);
        assert_eq!(beams2[0], 0.5);
        assert_eq!(beams2[1], 10.0);
        assert_eq!(beams3[23], 10.0);
        assert_eq!(beams3[0], 0.5);
        assert_eq!(beams3[1], 10.0);
        assert_eq!(beams4[23], 10.0);
        assert_eq!(beams4[0], 0.5);
        assert_eq!(beams4[1], 10.0);
        assert_eq!(beams5[23], 10.0);
        assert_eq!(beams5[0], 0.5);
        assert_eq!(beams5[1], 10.0);
        assert_eq!(beams6[23], 10.0);
        assert_eq!(beams6[0], 10.0);
        assert_eq!(beams6[1], 0.5);
    }

    #[test]
    fn test_eps() {
        let mut beams = [0.0; NUM_BEAMS];
        beams[22] = 3.0;
        beams[23] = 2.998;
        beams[0] = 3.0;
        beams[1] = 3.0;
        beams[2] = 3.0;
        beams[3] = 3.0;
        assert_eq!(expert(&beams).linear, VMAX);
        assert_eq!(expert(&beams).angular, 0.0);
   }

    #[test]
    fn test_class_to_angular() {
        for k in 0..NUM_ANGULAR_CLASSES {
            let mut beams = [1.0;24];
            let idx = (18 + k+2) % NUM_BEAMS;
            beams[(NUM_BEAMS + idx-1)%NUM_BEAMS] = 5.0;
            beams[(NUM_BEAMS + idx)%NUM_BEAMS] = 5.0;
            beams[(NUM_BEAMS + idx+1)%NUM_BEAMS] = 5.0;
            assert_eq!(expert(&beams).angular, class_to_angular(k).unwrap(),"k = {}", k);
        }
    }
}
