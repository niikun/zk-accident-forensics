//! LiDAR観測 → 速度指令の方策。ロボットのノードとSP1 guestで同一コードを共有する。
#![no_std]

/// 方策に入力するLiDARビーム数（/scanをダウンサンプルした値）
pub const NUM_BEAMS: usize = 24;

/// 速度指令（固定小数点化は学習後に置き換える）
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action {
    pub linear: f32,
    pub angular: f32,
}

/// 仮のルールベース方策（エキスパート）。前方が近ければ旋回、空いていれば直進。
pub fn expert(beams: &[f32; NUM_BEAMS]) -> Action {
    let front = beams[0].min(beams[1]).min(beams[NUM_BEAMS - 1]);
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
        let mut b = [3.0; NUM_BEAMS];
        b[0] = 0.3;
        assert_eq!(expert(&b).linear, 0.0);
    }
}
