//! /scan を購読し、方策の出力を /cmd_vel に出す。
use anyhow::Result;
use policy::{Action, preprocess_beams, expert};
use rclrs::*;
use ros_env::*;

fn main() -> Result<()> {
    let context = Context::default_from_env()?;
    let mut executor = context.create_basic_executor();
    let node = executor.create_node("policy_node")?;

    let cmd_pub = node.create_publisher::<geometry_msgs::msg::Twist>("cmd_vel")?;
    let _scan_sub = node.create_subscription::<sensor_msgs::msg::LaserScan, _>(
        "scan",
        move |scan: sensor_msgs::msg::LaserScan| {
            let a = match preprocess_beams(&scan.ranges) {
                Ok(beams) => {
                    expert(&beams)
                },
                Err(e) => {
                    eprintln!("preprocess failed : {e}");
                    Action {linear:0.0, angular: 0.0}
                }
            };
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
