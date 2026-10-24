# zk_accident_forensics

ロボットの行動を、どう証明するか。
Physical AI × ゼロ知識証明 — Rust・ROS 2・SP1 zkVMによる実装。
Physical AI 応用1講座 最終課題。

自律移動ロボット（差動二輪＋2D LiDAR）を、模倣学習で作ったMLPの方策で走らせる。
ロボットの機体（SDF）・走らせる「8の字」の部屋・走行デモ・方策のモデルは、すべて自作した。
方策の推論コードは `no_std` のRust crateにまとめ、ROS 2ノードとSP1 zkVMのguestで同じコードを使う。
レポートは [REPORT.md](REPORT.md)。

## 成果物

- 動画: <https://youtu.be/AuZTmBOkUqY>（走行シーンは2倍速）
- コード: <https://github.com/niikun/zk-accident-forensics>
- レポート: [REPORT.md](REPORT.md)

## 使用技術

| 分野 | 技術 |
|---|---|
| 言語 | Rust 1.97（`rust-toolchain.toml` で固定） |
| ロボット | ROS 2 Lyrical（Rustクライアント `rclrs` 0.8）、`ros_gz_bridge` |
| シミュレーション | Gazebo（gz-sim 10）、自作SDF（差動二輪＋2D LiDAR） |
| 学習 | Burn 0.22（模倣学習のMLP）、`csv`、データ整理に Python（pandas・mcap） |
| 推論 | `policy` crate（`no_std`・依存ゼロ、重みは定数として埋め込み） |
| ゼロ知識証明 | SP1 zkVM（guestで `policy::predict` を実行） |
| 環境 | pixi（RoboStack の ROS 2 を含む。`pixi.toml`／`pixi.lock`） |

## 開発の進め方（AIの使い方）

- 自分で行った: コードの実装、環境構築、実験、方針の判断
- Claudeに相談した: 設計の助言、調べもの、コードレビュー、つまずきの切り分け
- Claudeが作成した: レポートのたたき台、README、動画の編集（スライドと合成音声）

## できていること

- Gazebo上の自作ロボット・「8の字」の部屋で、学習した方策が壁にぶつからず走る
- ルールベースのエキスパートでデモを収集し、Burnで学習（angular 9クラス／mode 3クラス／減速量の回帰）
- 学習済みの重みを `policy` crateに埋め込み、`no_std` で推論（Burnの出力と一致を確認）
- SP1 guestで `policy::predict` が最後まで実行できる（サイクル数は1ステップ約200〜250万）

未実施: ハッシュチェーンによるログのコミットメント、証明の生成と検証、事故の判定。

## 構成

| 場所 | 役割 |
|---|---|
| `crates/policy` | 前処理・MLP推論・エキスパート（`no_std`、ROS非依存） |
| `crates/robot_nodes` | ROS 2ノード（`policy_node`: `/scan` → `/cmd_vel`）、デモの読み込み（`read_data`） |
| `crates/trainer` | Burnによる学習・評価・重みの書き出し |
| `worlds/` | Gazeboのワールドとロボット（SDF） |
| `config/` | `ros_gz_bridge` の設定 |
| `datas/` | 学習・評価用のCSV |
| `scripts/` | bagのCSV書き出しなど |

## 環境

pixi（RoboStack の ROS 2 Lyrical）で構築する。実行は `pixi run ...` 経由で、apt版の `setup.bash` は読み込まない。

```sh
pixi install --locked
pixi run cargo build
pixi run cargo test -p policy
```

## 走らせる

ターミナルを3つ使い、プロジェクト直下で次の順に実行する。

```sh
pixi run sim      # Gazebo（GUIなしは sim-headless）
pixi run bridge   # /scan, /cmd_vel のブリッジ
pixi run policy   # policy_node（ビルドも行う）
```

`cmd_vel` の確認: `pixi run ros2 topic echo /cmd_vel`

## 学習

```sh
pixi run cargo run -p trainer --bin trainer
```

学習データは `datas/train.csv`、評価データは `datas/test.csv`。成果物は `datas/artifact/`（git管理外）。
