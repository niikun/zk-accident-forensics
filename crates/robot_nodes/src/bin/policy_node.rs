//! /scan を購読し、方策の出力を /cmd_vel に出す。
use anyhow::Result;
use policy::{NUM_BEAMS, preprocess_beams, expert};
use rclrs::*;
use ros_env::*;

/// 360度スキャンを NUM_BEAMS 本に間引く（inf/NaN は range_max で置換）
fn downsample(scan: &sensor_msgs::msg::LaserScan) -> [f32; NUM_BEAMS] {
    let mut beams = [scan.range_max; NUM_BEAMS];
    let n = scan.ranges.len();
    if n == 0 {
        return beams;
    }
    for (i, b) in beams.iter_mut().enumerate() {
        let r = scan.ranges[i * n / NUM_BEAMS];
        if r.is_finite() {
            *b = r;
        }
    }
    beams
}

fn main() -> Result<()> {
    let context = Context::default_from_env()?;
    let mut executor = context.create_basic_executor();
    let node = executor.create_node("policy_node")?;

    let cmd_pub = node.create_publisher::<geometry_msgs::msg::Twist>("cmd_vel")?;
    let _scan_sub = node.create_subscription::<sensor_msgs::msg::LaserScan, _>(
        "scan",
        move |scan: sensor_msgs::msg::LaserScan| {
            let a = expert(&preprocess_beams(&scan.ranges.as_slice(), 30.0).unwrap());
            let mut twist = geometry_msgs::msg::Twist::default();
            twist.linear.x = a.linear as f64;
            twist.angular.z = a.angular as f64;
            if let Err(e) = cmd_pub.publish(&twist) {
                eprintln!("publish failed: {e}");
            }
        },
    )?;

    executor.spin(SpinOptions::default()).first_error()?;
    Ok(())
}
