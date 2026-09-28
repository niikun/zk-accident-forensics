# zk_accident_forensics

作業を始める前に必ず `HANDOFF.md` を読むこと（目的・締切・決定事項・環境のハマりどころ・次のタスク）。

- ビルド: `pixi run cargo build`（pixi＋RoboStackのLyrical、rclrs 0.8 / ros-env 0.3、colcon不要）。実行も`pixi run ...`経由。apt版の`setup.bash`はsourceしない
- `crates/policy` は `no_std`・ROS非依存を保つ（SP1 guestと共有するため）
- 進捗が変わったら `HANDOFF.md` の「現在の状態」「次にやること」を更新する
