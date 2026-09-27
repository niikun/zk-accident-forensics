# zk_accident_forensics

秘密を守ったまま、事故の真相を証明するロボット。
Physical AI 応用1講座 最終課題（事故フォレンジック × ZK）。

- ロボット側: ROS 2 Lyrical + Rust（`rclrs` 0.8）
- 方策: LiDAR → 速度指令の小さなMLP（模倣学習）。ノードとSP1 guestで同一コードを共有
- 証明: SP1で「ログ完全性・忠実な実行・事故の主張」を生データ・モデル非開示のまま証明（予定）

## 構成

| crate | 役割 |
|---|---|
| `crates/policy` | 方策本体（`no_std`、ROS非依存。現在は仮のルールベース） |
| `crates/robot_nodes` | ROS 2ノード。`policy_node`: `/scan` → `/cmd_vel` |

## ビルドと実行

Lyricalは主要メッセージ（`sensor_msgs`など）のRustバインディングを同梱しているため、
ROS環境をsourceすれば `cargo` だけでビルドできる。

```sh
source /opt/ros/lyrical/setup.bash
cargo build
cargo test -p policy
./target/debug/policy_node
```

動作確認（別ターミナル）:

```sh
ros2 topic echo /cmd_vel
ros2 topic pub -t 3 /scan sensor_msgs/msg/LaserScan "{range_max: 10.0, ranges: [0.3, 3.0, ...]}"
```

## メモ

- `rclrs` 0.7 は Lyrical 非対応（`Unsupported ROS distribution`でビルド失敗）。0.8 以上を使う
- `ros-env` は `rclrs` 0.8 が使う 0.3 に揃える
