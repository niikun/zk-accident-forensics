# HANDOFF — zk_accident_forensics

最終更新: 2026-09-29
このリポジトリで作業を始めるセッション向けの引き継ぎ。まずこのファイルを読むこと。

## 0. 進め方の原則（最優先）

このプロジェクトはユーザーの**学習が目的**で、完成させること自体が目的ではない。

- **コードはユーザーが書く**。Claudeは原則としてコードを書かない
- Claudeは**リサーチと助言**でユーザーを導く。答えを渡すより、考え方・手順・調べる場所・問いを示す
- **環境構築はユーザーが行う**。Claudeは調査と手順の提案まで
- **GitHubへのpushはユーザーが行う**

## 1. これは何か

東大 松尾・岩澤研「Physical AI 応用1」講座の**最終課題**。目標は**優秀賞**。

- 課題名: AI×ロボティクスによる創造的タスクの実現
- **締切: 2026-11-02（月）AM10:00**（提出目標は 11/1）。提出受付の開始は 10/8
- 提出物: 動画（MP4、1〜5分、倍速なら明記）＋コード一式（GitHub URL）＋簡単なレポート（概要・目的・工夫点）。omnicampusでZip＋GitHub URLを提出
- 必須条件: AIを何らかの形で使うこと（推論のみで可）。シミュレーションで可、ツールは自由
- 合格ラインは「ロボットがタスクを実行する動画」。加点は**創造性・技術力・実用性・表現力**

## 2. コンセプト（決定済み）

**「秘密を守ったまま、事故の真相を証明するロボット」＝事故フォレンジック×ZK**

自律移動ロボット（差動二輪＋LiDAR）が、模倣学習で作った方策（LiDAR→速度指令のMLP）で走る。

- **ROS2トピックの秘匿**: 生の観測・行動は公開せず、`blackbox_node`がハッシュチェーンの
  コミットメントだけを`/blackbox/commitment`に流す。生ログは暗号化してローカルに保存
- 事故後、運用者はモデルの重みも生ログも見せずに、SP1で以下を証明する
  - **S1 ログ完全性**: 提出ログが、走行中に公開したコミットメント列と一致する
  - **S2 忠実な実行**: 各時刻の行動が、登録済みモデル（重みのハッシュを公開）の出力と一致する
  - **S3 事故の主張**: 例「停止距離より内側に障害物が突然出現し、AIは直後に制動指令を出していた」
- 技術的な見せ場: **方策の推論コードをROS2ノードとSP1 guestで共有**する。ロボットが実際に
  動かしたコードそのものを証明している、と言える。そのため推論は固定小数点の純Rustで書く
- 実用性の語り所: ZKは無罪証明の道具ではない。「避けられなかった」も「AIの過失」も正直に出る

### なぜZKか（監査機関方式との違い）

- ログは**企業秘密（モデル・挙動）と個人のプライバシー**という二重の理由で開示できない
- 守秘義務付きの第三者監査と違い、**生データを誰にも預けずに済み、誰でも（被害者・保険会社・裁判所）自分で検証できる**
- 見せ方: 問題（説明責任 vs 秘匿）から語り、ZKは手段として後から出す。前半は「検証可能な計算」「秘密を守ったまま証明」と言い換える
- 限界として明記する: センサー入力そのものの真正性は保証しない（コミットは取得後のデータに対して行う）

### 運用者の恣意を排除する設計（2026-09-28決定）

- **証明の対象は事故の前後の区間に限定する**
- **区間は公開ルールで機械的に決める**: 走行中に衝突検知イベントをコミットし、その時刻を基準に固定の窓（例: 衝突の2秒前〜0.5秒後）を取る。運用者は区間を選べない
- **判定はguestが計算する**: 運用者に主張を選ばせない。公開済みの判定関数が区間のデータから「不可避」か「AI過失」かを出力する。運用者が選べるのは証明を出すかどうかだけ
- 区間内で抜けや差し替えがないことはハッシュチェーン（S1）で保証する
- 未決事項: 衝突の検知方法（contactセンサーか、LiDARの最短距離のしきい値か）、「不可避」の定義と、そのパラメータ（最大減速度・反応時間など）を公開・固定する場所

### 動画シナリオ（約3分）

問題提起 → 通常走行（A） → 飛び出しで衝突し「不可避」を証明（B） → ログ改ざん・モデルのすり替えを検証が弾く（C、赤×） → 仕組み図 → 応用。
余力があれば、バグのある旧モデルで衝突し「AI過失」が証明される場面（B'）を入れる。

### 先行研究と新規性（2026-09-27の調査、9/28に記録）

調査はユーザーが外部のAIで実施したもの。**Hello ZK Robotの実在（最終更新2026-02-25）以外は未確認**なので、レポートで引用する前に一次資料を開いて確かめること。

| 系統 | 代表例 | 今回との関係 |
|---|---|---|
| 事故調査用のブラックボックス | Winfield & Jirotka「The case for an ethical black box」(2017)、Ethical Black Boxのドラフト標準 [arXiv:2205.06564](https://arxiv.org/abs/2205.06564)、RoboTIPSの模擬事故調査（外骨格 [arXiv:2411.14008](https://arxiv.org/abs/2411.14008)） | 問題設定の出発点。「記録して調査する」は既存。ログを見せられない場合にZKへ、と話をつなぐ |
| 暗号化＋改ざん検知ログ | [Vouch PAD-069 Confidential, Tamper-Evident Robot Black-Box](https://github.com/vouch-protocol/vouch/blob/main/docs/disclosures/PAD-069-confidential-tamper-evident-robot-blackbox.md)（AES-GCM＋ハッシュチェーン＋署名） | **S1とほぼ同じ**。S1は基盤技術と割り切る |
| ZKでロボットの安全性を証明 | [Hello ZK Robot](https://github.com/Inversed-Tech/hello-zk-robot)（SP1、非公開の軌跡に対して範囲・禁止区域・速度を検査、`run_hash`で証明を特定の走行に結びつける） | **S1＋S3の原型**。ただし証明するのは「安全制約を満たした」まで |
| ROS 2＋zkML | [JOLT Atlas zkML Guard](https://github.com/hshadab/robotics)（モデルハッシュ・入力ハッシュ・推論結果・証明を結びつけ、`/cmd_vel`の許可に使う） | **S2に近い**。S2単独を新規性にしない |
| 秘匿したままの認識・判断の証明 | Hermes Seal [arXiv:2603.26343](https://arxiv.org/abs/2603.26343)（自動運転） | 「企業秘密を守ったまま検証」という動機が同じ |
| 推論の来歴の証明 | IETF draft-mw-spice-inference-chain | 推論の来歴 → 行動の来歴 → 事故の来歴、と話を広げられる |
| 事故報告＋ZKP | TAR-PZKP（車両事故報告の伝送、PUF＋ZKP） | 伝送の認証が主で、事故原因の判定はしない |

**新規性として押し出す点**（調査した範囲では一致するものがなかった）
- 事故区間を公開ルールで機械的に決める
- 「不可避/AI過失」の判定関数を公開し、guestで計算する（運用者に主張を選ばせない）
- 実際のROS2ロボットの走行ログと、その方策の実行（ノードとguestでコードを共有）をzkVMで結びつける
- 呼び方の案: 「秘密を保持した検証可能なロボット事故フォレンジック（Privacy-Preserving Verifiable Robot Accident Forensics）」。
  流れは Commit → 事故区間の決定 → 方策の再計算 → 原因判定の証明 → 検証

**限界として明記する**: ZKが保証するのは「公開ルールを秘密ログに正しく適用した」ことまで。ルールのパラメータ（最大減速度など）が妥当かどうかと、センサー入力が本物かどうかは保証しない（参考: [ROS 2 Threat Model](https://design.ros2.org/articles/ros2_threat_model.html)）。

**TODO**: 先行研究との差分を1枚の表にして、レポートの工夫点の核にする（W4〜W5）

## 3. 現在の状態（2026-09-29）

- 初回コミット済み: `crates/policy`（no_std、仮のルールベース`expert()`）と`crates/robot_nodes`（`policy_node`）
- `policy_node`: `/scan`を24本に間引いて方策に渡し、`/cmd_vel`（`geometry_msgs/Twist`）に出す
- 疎通確認済み: `ros2 topic pub`で前方0.3mの偽スキャンを流すと`angular.z=0.8`（停止・旋回）が返る
- `cargo test -p policy`はテスト1件（`turns_when_blocked`）
- **元のPCでpixi版の疎通を確認済み**（2026-09-28）: `rust-toolchain.toml`（1.97.0＋rustfmt/clippy）、`pixi.toml`（robostack-lyrical＋conda-forge、`ros-lyrical-ros-base`）で
  `pixi run cargo build` → `policy_node`が動き、偽スキャン（`ranges: [0.3, 3.0, 3.0, 3.0]`）で`angular.z=0.8`が返る
- **元のPCでpixi版Gazeboの表示を確認済み**（2026-09-29）: `ros-lyrical-ros-gz`を追加し、`pixi run gz sim shapes.sdf`がWSLgで起動。RTFは約70%
- `pixi.toml`/`pixi.lock`はコミットしてpush済み（`7bb895e`）
- **サブPCでも同じ環境を再現できた**（2026-09-29）: `pixi install --locked`、ビルド、`policy_node`の疎通（`z: 0.8`）、Gazeboの表示（RTF 70%超、元のPCと同程度）。残りは秘書ノートの同期だけ（6章の0-4-7）
- **SDFの自作を開始**（2026-09-29、サブPC）: `worlds/forensics.sdf`（お手本のコピー、`vehicle_green`はコメントアウト、構文チェック済み、未コミット）。次は6章の2
- 疎通確認のやり方（ターミナル3つ、すべてプロジェクト直下で）:
  1. `pixi run ./target/debug/policy_node`
  2. `pixi run ros2 topic echo /cmd_vel`
  3. `pixi run ros2 topic pub -r 1 /scan sensor_msgs/msg/LaserScan "{range_max: 10.0, ranges: [0.3, 3.0, 3.0, 3.0]}"` → 2に`z: 0.8`が出れば成功

## 4. 環境の事実（ハマりどころ）

### 2台のPCと環境の統一方針（2026-09-28決定、移行中）

- 作業PCは2台。**元のPC**（Ubuntu 26.04、apt版Lyrical、Rust 1.97。HANDOFFを書いた環境）と、
  **サブPC**（Ubuntu 24.04、ROS未導入、既定のstableは1.96だが1.97のツールチェーンはある。
  SP1の`cargo-prove`は2026-06-25ビルド、ツールチェーンは`succinct`）
  - ハード: Ryzen 5 5625U、**GPUはAMD Radeonの内蔵のみ（NVIDIAなし）**、Windows全体の物理RAMは16GB
  - `.wslconfig`は`memory=12GB, swap=4GB`（WSLから見えるのは約11.7GB）。Windows自体に3〜4GB要るので、**WSLに15GBは割り当てられない**（上限は12〜13GB）
- Lyricalは24.04ではTier3でaptのバイナリがないため、**両PCとも pixi + RoboStack（`https://prefix.dev/robostack-lyrical`）に統一する**
  - `pixi.toml`/`pixi.lock`をコミットし、もう片方は`pixi install`で同じ環境を作る
  - robostack-lyricalの`ros2-*`パッケージにも`share/<pkg>/rust`のバインディングが同梱されている（確認済み）。`ros-lyrical-*`は中身が空のエイリアス
  - apt版Lyricalの`setup.bash`をsourceしない（pixi環境と混ざる）。conda baseの自動有効化にも注意
  - ✅ 元のPCでは、pixi版Gazebo（`ros-lyrical-ros-gz` 3.0.10、gz-sim 10 = Jetty）のGUIがWSLgで表示できた（`pixi run gz sim shapes.sdf`、2026-09-29）。
    サブPCで駄目なら、Ubuntu 26.04のディストリを追加してaptで入れる
- Rustの版は`rust-toolchain.toml`で1.97.0に固定する
- **pixiのハマりどころ（元のPCで確認）**
  - チャンネルは`"https://prefix.dev/robostack-lyrical"`とURLで書く。短い名前だと`conda.anaconda.org`を見に行って404になる。順番はrobostackが先、conda-forgeが後（strict priority）
  - **pixiは`LD_LIBRARY_PATH`を設定しない**（condaはRPATHで解決する流儀）。cargoで作ったバイナリは`pixi run`の中でも`librcl.so: cannot open shared object file`で落ちる。
    `[activation.env]`に`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "-C link-arg=-Wl,-rpath,$CONDA_PREFIX/lib"`を書いてRPATHを埋め込む
    （`LD_LIBRARY_PATH`で解決しないのは、Gazeboの描画がWSLgのGPUドライバとぶつかるのを避けるため。ターゲットを限定するのはSP1 guestのビルドに混ぜないため）
  - RPATHは絶対パスなので、バイナリはPCごとに`pixi run cargo build`で作る。apt版からの移行時は最初に`cargo clean`
  - 実行はすべて`pixi run ...`経由（例: `pixi run ros2 topic echo /cmd_vel`）
- **Gazeboの描画（元のPC、未確定）**
  - pixi環境にはlibglvnd（OpenGLの振り分け役）しか入らず、Mesaのドライバは入らない。描画はUbuntu側のMesa（`d3d12`ドライバ＝WSLg経由のGPU）に回る
  - RTFが約70%。`/usr/lib/wsl/lib`にIntel用とNVIDIA用の両方があるため、**Intelの内蔵GPUで描画している可能性**がある
  - 確認方法: `sudo apt install mesa-utils` → `glxinfo -B | grep -E "renderer|Device"`（D3D12 (NVIDIA …) / D3D12 (Intel …) / llvmpipe=CPU描画）
  - Intelだった場合は`MESA_D3D12_DEFAULT_ADAPTER_NAME=NVIDIA`で切り替えられるか試し、効けば`[activation.env]`に追加する
  - 急ぎではない。デモ収集はGUIなし（`gz sim -s`）で回せる
- 秘書ノート（`~/company`、`niikun/company`）はgitで同期する。サブPCは7/19で止まっているのでpullが必要

### 元のPCの事実（apt版。pixi移行後は参考）

- OS: WSL2（Ubuntu 26.04）、ROS 2 **Lyrical**（`/opt/ros/lyrical`）、rustc/cargo 1.97
- マシン: 12コア、RAM 15GB、GPU Quadro T1000
- **`rclrs`は0.8以上を使う**。0.7はLyrical非対応で、`Unsupported ROS distribution`というコンパイルエラーになる
- **`ros-env`は0.3**（rclrs 0.8と揃える）。メッセージは`use ros_env::*;`で`sensor_msgs::msg::LaserScan`などが使える
- Lyricalは主要メッセージのRustバインディングを`/opt/ros/lyrical/share/<pkg>/rust`に同梱している。
  **colconもrosidl_rustのcloneも不要**。`source /opt/ros/lyrical/setup.bash`のあとに`cargo build`で通る
- `robot_nodes/package.xml`は将来colconで扱う場合に備えて置いてある（現状は使っていない）
- apt版のros_gzは未導入（pixi版で導入済みなので、apt版には入れない）
- 参考用の`~/project/ros2_rust_ws`（ros2_rustのソースとexamples）は別物。こちらには混ぜない

```sh
source /opt/ros/lyrical/setup.bash
cargo build && cargo test -p policy
./target/debug/policy_node
```

## 5. スケジュール

| 週 | ゴール |
|---|---|
| W1 9/28–10/4 | ✅ Rustノードの疎通 → ros_gzを導入し、差動二輪＋LiDARのロボットをWSLgで表示 → ルールベース走行（10/1は第5回ROS2講義） |
| W2 10/5–11 | エキスパートでデモ収集 → BurnでMLPを学習 → 固定小数点推論を`policy`に実装 → 学習済み方策で走行。**ここで動画を1本撮り、合格ラインを確保** |
| W3 10/12–18 | `blackbox_node`（ハッシュチェーン・コミットメント公開・暗号化ログ）、SP1でS1+S2 |
| W4 10/19–25 | S3（事故の主張）、シナリオB/C、検証結果の可視化（✅/❌） |
| W5 10/26–11/1 | 動画・レポート・README仕上げ、提出 |

## 6. 次にやること（W1の残り）

0. 環境をpixiに統一する（4章。ユーザーが実施）
   1. 元のPC: 秘書ノートをpush、✅ `rust-toolchain.toml`を追加
   2. ✅ 元のPC: pixiを導入し`pixi init` → `pixi add ros-lyrical-ros-base` → `pixi run cargo build`、`ros2 topic pub`での疎通をapt版と比較（4章のハマりどころを参照）
   3. ✅ `pixi add ros-lyrical-ros-gz`でGazeboのGUI表示を確認し（元のPC）、`pixi.toml`/`pixi.lock`/`.gitignore`（`.pixi/`）をコミット
   4. **サブPC（次はここから）**:
      1. 事前確認: `cc --version`（Rustのリンクに必要。無ければ`build-essential`）、ディスクの空き（`.pixi/`は数GBになる）
      2. `git pull`（未cloneなら`git clone git@github.com:niikun/zk-accident-forensics.git`）
      3. ✅ pixiを導入（pixi 0.81.0）→ `pixi install --locked`（lockは書き換わらず、`ROS_DISTRO=lyrical`、`share/sensor_msgs/rust`あり、2026-09-29）
      4. ✅ `rustc --version`が1.97.0（`rust-toolchain.toml`で自動切り替え）
      5. ✅ `pixi run cargo build`で`policy_node`をビルド（2026-09-29）→ 3章の手順で疎通確認し、`z: 0.8`が返った
      6. ✅ `pixi run gz sim shapes.sdf`でGazeboが表示され、RTFは70%超（2026-09-29。サブPCはAMD内蔵GPUのみなので、描画はD3D12 (AMD Radeon)のはず）
      7. 秘書ノート（`~/company`）を`git pull`（7/19で止まっている）
1. ✅ ros_gzを導入する（0-3で導入済み）
2. **（作業中）** 差動二輪＋LiDARのロボットと障害物のあるワールドを`worlds/`に置く。**SDFは自作する**（2026-09-29決定。
   TurtleBot3は使わない。LiDARのビーム数・車輪間隔・速度などを自分で把握し、S3の公開パラメータに直結させるため）
   - ✅ gz-simのお手本`diff_drive.sdf`を`worlds/forensics.sdf`にコピーし、`vehicle_green`をコメントアウトした。`gz sdf -k`は`Valid.`（サブPC）
   - ⬜ 起動して、`gz topic -t /model/vehicle_blue/cmd_vel -m gz.msgs.Twist -p 'linear: {x: 0.3}'`で青い車が進むか確認
   - ⬜ `worlds/`をコミット（まだgitで追跡されていない）
   - ⬜ `vehicle_green`を削除する（XMLのコメントは入れ子にできないので、コメントアウトのまま育てない）
   - ⬜ ワールドにSensorsシステム（`<render_engine>ogre2</render_engine>`）を足し、車体に`gpu_lidar`を付け、障害物（box）を置く
   - ⬜ `gz topic -e -t <lidarのトピック>`で値が出るか確認
3. `ros_gz_bridge`で`/scan`と`/cmd_vel`をブリッジし、`policy_node`で走らせる（設定は`config/bridge.yaml`に置く案）
4. 起動手順を`pixi.toml`の`[tasks]`にまとめる（両PCで同じコマンドで起動できる）

### SDF作りのメモ

- お手本（`.pixi/envs/default/share/`の下。**直接編集しない**。`pixi install`で上書きされ、gitにも入らない）
  - `gz/gz-sim/worlds/diff_drive.sdf`: 車体・車輪・キャスターの組み方、`gz-sim-diff-drive-system`
  - `gz/gz-sim/worlds/visualize_lidar.sdf`: `gpu_lidar`の書き方と、ワールドに要るシステム（Physics、Sensors＋`ogre2`、SceneBroadcaster）
  - `ros_gz_sim_demos/config/diff_drive.yaml`、`gpu_lidar.yaml`: bridgeのYAMLの書き方
- 構成: 最初はワールドとロボットを`worlds/forensics.sdf`の1ファイルに書く。育ったら`models/robot/model.sdf`に分けて`<include>`する
- **ハマりどころ: ビーム0の向き**。`policy::expert()`（`crates/policy/src/lib.rs:16`）は`beams[0]`が真正面、`beams[1]`と`beams[23]`がその両隣という前提。
  gzのlidarを`min_angle=-π, max_angle=π`にすると`ranges[0]`は真後ろを向く。SDFの角度範囲で合わせるか、`downsample()`で回転させるかはユーザーが決める
  （方策の入力の定義になり、SP1 guestと共有する）。`samples`は24の倍数にし、360度で始点と終点が重複しないようにする
- **SDFの数値はS3の公開パラメータになる**ので、決めたら表にして残す
  - `max_linear_velocity`、`min_linear_acceleration`（最大減速度） → 停止距離
  - LiDARの`update_rate` → 方策の周期（反応時間の下限）
  - LiDARの最大距離と、障害物が「突然出現する」距離の関係
  - 今の`expert()`（0.2m/sで直進、前方0.5m未満で旋回）が、SDFの加速度制限と矛盾しないか
- 考えておく問い: `cmd_vel`のトピック名はなぜ`/model/vehicle_blue/...`になるのか。ROSの`/cmd_vel`と、SDFの`<topic>`とbridgeのYAMLのどちらで名前を合わせるか

### サブPCでROSなしでできること（W3の予習）

- **Hello ZK Robotを動かす**（SP1 SDK 6.0.1、guestは80行、hostは121行）。`~/rust/projects/hello-zk-robot`にcloneする（このリポジトリには混ぜない）
  1. `cd script && RUST_LOG=info cargo run --release -- --execute --run ../examples/sample_run.json`（実行のみ）
  2. 同じコマンドを`--prove`で実行。RAM 11GBで証明を作れるかを測る（落ちたら`.wslconfig`の`memory=`を上げる）
  3. 改ざん実験（シナリオCの予行）: 点を1つ禁止区域に動かす → `ok=false`になるか。`run_hash`だけを書き換える → 弾かれるか
- 読みながら考える問い
  - privateとpublicの区別はどこで行われているか。今回は何をprivateにするか
  - `run_hash`は軌跡全体に対するSHA-256 1回。走行中にコミットを少しずつ公開するなら、なぜハッシュチェーンが必要か
  - 安全チェックに失敗したとき、証明は作られるのか、panicするのか（`ok=false`の証明も必ず出せることが「主張を選ばせない」設計の前提）
  - `VMAX`などのパラメータは誰がどこで固定しているか（未決事項「不可避のパラメータを公開・固定する場所」のヒント）
- 結果（サイクル数、証明時間、ピークメモリ）を記録し、事故前後の数十ステップを証明できるか見積もる

## 7. 設計方針・予定のcrate

- `policy`: `no_std`でROSに依存させない（SP1 guestから使うため）。学習後は重みを埋め込んだ固定小数点のMLPに置き換える。`expert()`はデモ収集用として残す
- `robot_nodes`: ROS2ノード群。`policy_node`、今後`blackbox_node`や障害物（歩行者）を動かすノードを追加
- 今後追加予定: 学習用crate（Burn）、SP1一式（`program/`=guest、`script/`=host、`lib/`=共有型）。
  SP1は`cargo prove new --bare`のテンプレートの流儀に合わせる（参考: `~/project/wood_zk_traceability`、`~/project/mpc_group_purchasing`）
- 証明は**事後にオフラインで作る**（commit-now-prove-later）。走行中はコミットだけ行う

## 8. リスクと保険

- Gazeboが重い・WSLgで表示できない場合 → Rust製の2Dシミュレータ＋rerunで可視化に切り替える（課題は他ツール可）
- RAMは元のPCで15GB、サブPCで約12GB（増やせない）。SP1はcore/compressed証明まで。証明する区間は事故前後の数十ステップに絞る。Groth16ラップやオンチェーン検証はやらない
  - **本番の証明は元のPCで作る**。サブPCは実行（`--execute`）とサイクル数の計測まで。サブPCで証明が必要なら、swapを増やして（例: `swap=16GB`）遅さを受け入れる
  - サブPCでHello ZK Robotの`--prove`が通るかを測り、必要メモリの目安にする
- 固定小数点化で精度が落ちたら、層を小さくするかビーム数を減らす
- 大原則: **W2末までに合格ラインの動画を確保**し、その後に加点要素を積む

## 9. 関連資料

秘書ノート（`~/company/.company/secretary/notes/`）:

- `topics/physical-ai-final-accident-forensics.md`: 計画の正本
- `topics/physical-ai-advanced1-course.md`: 講座情報と最終課題の要項
- `topics/zkml-robot-accident-verification.md`: 元アイデア
- `topics/physical-ai-zk-integrated-roadmap.md`: 統合ロードマップ（本課題でPhase A-2とPhase B M3-M4を前倒しする）
- `2026-09-28-decisions.md` / `2026-09-28-learnings.md`: 今日の決定と学び

## 10. 作業ルール

- コミットメッセージは日本語でよい。末尾に`Co-Authored-By`を付ける
- 決定や学びが出たら、秘書ノート（上記）にも記録する
- 進捗が変わったら、このHANDOFFの「3. 現在の状態」と「6. 次にやること」を更新する
