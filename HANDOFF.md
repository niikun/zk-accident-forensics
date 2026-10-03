# HANDOFF — zk_accident_forensics

最終更新: 2026-10-03
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

## 3. 現在の状態（2026-10-03）

- 初回コミット済み: `crates/policy`（no_std、仮のルールベース`expert()`）と`crates/robot_nodes`（`policy_node`）
- `policy_node`: `/scan`の生の360本を`policy::preprocess_beams()`で24本にして`expert()`に渡し、`/cmd_vel`（`geometry_msgs/Twist`）に出す。
  `preprocess_beams`が`Err`（長さが360でない）なら**停止指令（0, 0）を出し**、エラーを表示する（2026-10-03、`b811d5d`）
- 疎通確認済み（2026-10-03）: 360本の偽スキャン（`ranges[0]`だけ0.3、他は3.0）で`angular.z=0.800000011920929`（停止・旋回）が返る。`ranges[30]`だけ0.3（`beams[2]`、前方の判定外）なら直進。
  端数は`f32`の0.8を`f64`に広げたため（バグではない。ノードとguestで結果を揃えるために固定小数点化する理由の一例）
- `cargo test -p policy`はテスト4件（`turns_when_blocked`、`test_inf_case`、`test_preprocess_beams`、`test_preprocess_beams_boundary`）。すべて通過（2026-10-03）
- **元のPCでpixi版の疎通を確認済み**（2026-09-28）: `rust-toolchain.toml`（1.97.0＋rustfmt/clippy）、`pixi.toml`（robostack-lyrical＋conda-forge、`ros-lyrical-ros-base`）で
  `pixi run cargo build` → `policy_node`が動き、偽スキャン（`ranges: [0.3, 3.0, 3.0, 3.0]`）で`angular.z=0.8`が返る
- **元のPCでpixi版Gazeboの表示を確認済み**（2026-09-29）: `ros-lyrical-ros-gz`を追加し、`pixi run gz sim shapes.sdf`がWSLgで起動。RTFは約70%
- `pixi.toml`/`pixi.lock`はコミットしてpush済み（`7bb895e`）
- **サブPCでも同じ環境を再現できた**（2026-09-29）: `pixi install --locked`、ビルド、`policy_node`の疎通（`z: 0.8`）、Gazeboの表示（RTF 70%超、元のPCと同程度）。残りは秘書ノートの同期だけ（6章の0-4-7）
- **SDFの自作を開始**（2026-09-29、サブPC）: `worlds/forensics.sdf`（お手本のコピー、`vehicle_green`はコメントアウト、構文チェック済み、`c8cb379`でコミット済み）。青い車が`cmd_vel`で進むことを確認。LiDARは低い位置に付け、そのために車体を小さく作り直す方針に決定（6章の2）
- **小型の車体が完成**（2026-09-30、サブPC）: `vehicle_blue`を小型に作り直し、`lidar_link`＋`gpu_lidar`を載せた（設計値は6章の2の表）。`gz sdf -k`は`Valid.`。
  GUIなしの計測で、前進0.300m/s・旋回0.500rad/sが指令どおりに出て、停止時のつんのめりもない（pitch 0度）。
  inertiaの最終修正（0.0001 / 0.00005）まで`1daf8f8`でコミット済み
- **LiDARが値を出すようになった**（2026-09-30、`1daf8f8`でコミット済み）: ワールドにSensorsシステムを追加し、`/lidar2`から`ranges`が届くことを確認
- **LiDARを360度（360本、`ranges[0]`＝正面）にした**（2026-09-30、`bf3b354`でコミット済み）。正面0.90006mで予想どおり。スキャンの変換仕様（`-inf`→0、`+inf`→`range_max`、`NaN`→0、区間の最小値、`policy` crateに置く）を決めた
- **スキャンの変換を`policy::preprocess_beams()`として実装した**（2026-10-02、`557c078`でコミット済み）。生の360本を24本にする。正面は`beams[0]`の区間の中央（`ranges[353..360]`＋`ranges[0..8]`）。
  `expert()`からは置き換え処理を外した。
- **`policy_node`を`preprocess_beams()`に切り替えた**（2026-10-03。呼び出しの置き換えは`ad9f7d8`、`Err`の処理と`RANGE_MAX`は`b811d5d`）。古い`downsample()`は削除し、`-inf`の穴は塞がった。
  `range_max`は引数をやめ、`policy::RANGE_MAX`（`pub const`、10.0＝SDFの`<max>`）に固定した（方策の入力の定義なので、ノードとguestで食い違わないようにするため）
- **障害物を置いた**（2026-09-30、`22bda7c`でコミット済み）: `model_with_lidar`を消し、x=1.0, y=2に静的な箱`box`（0.2×0.2×0.3）を置いた。車が箱にぶつかって止まる。
  衝突した状態でLiDARの正面は`-inf`になった（6章の2）。`policy_node`の`downsample()`が`-inf`を`range_max`（遠い）に変えてしまう穴が見つかった（2026-10-03に`preprocess_beams()`への切り替えで解消）
- **閉ループ走行を達成**（2026-10-03、未コミット）: Gazebo → `ros_gz_bridge`（`config/bridge.yaml`）→ `policy_node` → Gazebo。車は箱に向かって直進し、手前で旋回する。
  途中で「旋回し続ける」不具合が出て、原因は**重心が車軸のほぼ真上（3mm後ろ）で、前に倒れていた**こと。前にもキャスターを付けて直した（6章の2）。
  起動手順（ターミナル3〜4つ、プロジェクト直下、この順で）:
  1. `pixi run gz sim -r worlds/forensics.sdf`
  2. `pixi run ros2 run ros_gz_bridge parameter_bridge --ros-args -p config_file:=config/bridge.yaml`
  3. `pixi run ./target/debug/policy_node`
  4. （観察）`pixi run ros2 topic echo /cmd_vel`、`pixi run ros2 topic info /cmd_vel`（Publisher/Subscriptionが各1）
- 疎通確認のやり方（Gazeboなし、偽スキャン。ターミナル3つ、すべてプロジェクト直下で）:
  1. `pixi run ./target/debug/policy_node`
  2. `pixi run ros2 topic echo /cmd_vel`
  3. `pixi run ros2 topic pub -r 1 /scan sensor_msgs/msg/LaserScan "{range_max: 10.0, ranges: $(python3 scripts/dummy.py)}"` → 2に`z: 0.8`が出れば成功
     - `dummy.py`は360本のリストを`print`する（`[0.3, 3.0, ...]`はそのままYAMLのリストになる）。`$(...)`を展開させるため、全体はダブルクォートで囲む
     - Errの経路: `ranges: [0.3, 3.0, 3.0, 3.0]`（4本）を流すと、停止（0, 0）が返り、ノードは落ちずにエラーを出す（2026-10-03確認）

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
| W1 9/28–10/4 | ✅ Rustノードの疎通 → ✅ ros_gzを導入し、差動二輪＋LiDARのロボットをWSLgで表示 → ✅ ルールベース走行（閉ループ、10/3）（10/1は第5回ROS2講義） |
| W2 10/5–11 | エキスパートでデモ収集 → BurnでMLPを学習 → 固定小数点推論を`policy`に実装 → 学習済み方策で走行。**ここで動画を1本撮り、合格ラインを確保** |
| W3 10/12–18 | `blackbox_node`（ハッシュチェーン・コミットメント公開・暗号化ログ）、SP1でS1+S2 |
| W4 10/19–25 | S3（事故の主張）、シナリオB/C、検証結果の可視化（✅/❌） |
| W5 10/26–11/1 | 動画・レポート・README仕上げ、提出 |

## 6. 次にやること（W1の残り）

0. 環境をpixiに統一する（4章。ユーザーが実施）
   1. 元のPC: 秘書ノートをpush、✅ `rust-toolchain.toml`を追加
   2. ✅ 元のPC: pixiを導入し`pixi init` → `pixi add ros-lyrical-ros-base` → `pixi run cargo build`、`ros2 topic pub`での疎通をapt版と比較（4章のハマりどころを参照）
   3. ✅ `pixi add ros-lyrical-ros-gz`でGazeboのGUI表示を確認し（元のPC）、`pixi.toml`/`pixi.lock`/`.gitignore`（`.pixi/`）をコミット
   4. サブPC（0-4-7の秘書ノート以外は完了）:
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
   - ✅ 起動して、`gz topic -t /model/vehicle_blue/cmd_vel -m gz.msgs.Twist -p 'linear: {x: 0.3}'`で青い車が進むことを確認（2026-09-29、サブPC）
   - ✅ `worlds/`をコミット（`c8cb379`）
   - ✅ `vehicle_green`を削除した（2026-09-30）
   - ✅ **車体を小型に作り直した**（2026-09-30、サブPC）。LiDARを低く付けるため（実機でも膝の高さが多く、歩行者の脚や低い障害物が見える）。
     元のchassis（2.0×1.0×0.57m）にLiDARを入れると自分の車体を写すので、小さく・低くした。modelのpose zを0にして、各linkのz＝地面からの高さにしている

     | 部品 | 形・寸法 [m] | pose z | 地面からの範囲 [m] | mass [kg] | inertia |
     |---|---|---|---|---|---|
     | model `vehicle_blue` | — | 0（x=0, y=2） | — | — | — |
     | chassis | 箱 0.14×0.18×0.05（L×W×H） | 0.055 | 0.03〜0.08（床とのすき間c=0.03） | 0.8 | 0.00233 / 0.00147 / 0.00347 |
     | lidar_link | 箱 0.05角 | 0.105 | 0.08〜0.13（**スキャン面 約0.105**） | 0.1 | 0.0000416667 ×3 |
     | 左右の車輪 | 球 r=0.05、y=±0.09 | 0.05 | 0〜0.10 | 0.1 | 0.0001 ×3 |
     | キャスター | 球 r=0.05、x=-0.07 | 0.05 | 0〜0.10 | 0.05 | 0.00005 ×3 |
     | diff-drive | `wheel_separation` 0.18、`wheel_radius` 0.05 | | | | |

     - 計算式: 車輪・キャスターの中心z＝半径、chassisのz＝c＋H/2、LiDARのz＝c＋H＋h/2。箱のinertiaは`ixx=m(W²+H²)/12`、`iyy=m(L²+H²)/12`、`izz=m(L²+W²)/12`、球は`2/5·m·r²`
     - 計測（GUIなし、シミュレーション内の時刻で）: 前進0.300m/s、旋回0.500rad/s（指令どおり）、停止時のpitch 0度、roll 0度
     - 見た目だけの残り: 車輪（y=±0.09）の半分が車体に埋まっている。外に出すなら±0.14にして`wheel_separation`も合わせる
     - **学び（2026-09-30）**
       - `<pose>`は中心の位置。linkのzはmodelのzに足される。下端が地面にちょうど接するのは「中心の高さ＝半径」のとき
       - `<size>`/`<radius>`だけを変えると、pose・collision・inertia・diff-driveが大きな車のまま残る。**寸法を変えたら全部を計算し直す**
       - **diff-driveの`wheel_radius`が実際と違うと、速度が比率どおりにずれる**（0.3のままで実際は1/6の速度）。odometryも同じ間違った値で計算するので「走った」と嘘をつく。実際の位置は`/world/<world>/pose/info`で見る
       - **inertiaが大きすぎると、ブレーキで前につんのめる**（車輪のinertiaが約60倍で、停止時にpitch約31度＝車体の前が接地）。車輪の反トルクが重心の復元トルクを上回るため。キャスターが後ろだけなので前に支えがない。**massを変えたらinertiaも必ず計算し直す**
       - 起動中のGUIと別にテストするときは`GZ_PARTITION=<名前>`で通信を分ける（分けないと指令がGUIの車にも届く）
       - `gz topic -p`の最初の1回は接続前に送られて落ちることがある。数回送る
       - `gz`はpixi環境にしかない。`pixi run gz sdf -k worlds/forensics.sdf`のように実行する
   - ✅ ワールドにSensorsシステム（`gz-sim-sensors-system`＋`<render_engine>ogre2</render_engine>`）を追加した（2026-09-30、`1daf8f8`）
   - ✅ `pixi run gz topic -e -t /lidar2 -n 1`で値が出ることを確認（2026-09-30）。`angle_min/max`=±1.396263、`count`=640、`vertical_count`=1、`ranges`はすべて`inf`（障害物なし、床も写らない＝スキャン面が水平で正しい）
     - **学び**: シミュレーションが一時停止中（GUIで起動した直後）はLiDARが何も出さない。`/stats`の`paused: true`で確認できる。▶を押すか、`gz sim -r`で起動する
     - `intensities`は反射強度。`gpu_lidar`は材質の反射率を計算しないので全部0になる。方策では使わない。見るときは`| grep -E "ranges|count|angle_min|angle_max"`で絞る
     - `ranges[0]`は`angle_min`（−80度、**右端**）で、番号が増えるほど左に回る。640本なら真正面は319〜320番あたり
   - ✅ お手本の`model_with_lidar`を消し、車の正面（x=1.0, y=2）に静的な箱`box`（0.2×0.2×0.3、pose z=0.15、`<static>true</static>`、collisionとvisualは同じ寸法）を置いた（2026-09-30、`22bda7c`）。`gz sdf -k`は`Valid.`
     - **学び**: `<visual>`/`<collision>`は`<model>`直下に書けず、`<link>`の中に置く（model → link → visual/collision）。無いと`A model must have at least one link`
     - **学び**: `<static>`も`<collision>`も無い箱は、▶を押すと重力で落ち、地面をすり抜けて消える。`Valid.`は書式が正しいだけで、物理的に正しいことは保証しない
     - `<static>`だけだと車が箱をすり抜ける。衝突させるには`<collision>`も要る
   - ✅ 車が箱にぶつかって止まることを確認（2026-09-30）
   - ✅ **衝突した状態で正面のビームは`-inf`**（2026-09-30）。LiDAR（車体中心）から箱の面まで約0.07m（chassisの前端＝中心から0.07m）で、`<range><min>`の0.08m未満のため
     - `ranges`の範囲外の値（REP 117と同じ）: `-inf`＝近すぎる（min未満）、`+inf`＝何もない（max以内に当たらない）、`NaN`＝計測失敗
     - **穴**: [policy_node.rs](crates/robot_nodes/src/bin/policy_node.rs)の`downsample()`は`is_finite()`でない値をすべて`range_max`にする。`-inf`（張り付くほど近い）が「10m先まで空いている」に化け、一番危ない瞬間に直進を選ぶ
     - 衝突の検知をLiDARのしきい値で行うなら、しきい値は`range.min`より大きくするか、`-inf`そのものを衝突とみなす（未決事項の判断材料）
   - ✅ **LiDARを360度に変更し、並びを`expert()`の前提に合わせた**（2026-09-30、`bf3b354`）: `samples`=360、`min_angle`=0、`max_angle`=6.2657（＝2π×359/360、359度）。
     `ranges[0]`が正面で、番号が増えるほど左回り（REP 103と同じ）。1度刻みで始点と終点が重ならない。`downsample()`の`ranges[15*i]`は`beams[1]`＝左15度、`beams[23]`＝右15度になる
     - 確認（`gz topic -e -t /lidar2 -n 1`）: `count` 360、`angle_step` 0.0174532（1度）、`angle_max` 6.2657。正面`ranges[0]`＝0.90006（予想0.9m）。
       箱に当たるのは`[0]`〜`[6]`と`[354]`〜`[359]`の13本で左右対称。真横（90番・270番）は`inf`（スキャン面0.105mと車輪の上端0.10mの差5mmでも、自分の車輪は写らない）
     - **学び**: `angle_min`の行が出ないのは値が0だから（protobufは既定値の項目を表示しない）
     - **学び**: **SDFを直したらGazeboを起動し直す**。gzはSDFを起動時に1回しか読まない（再起動を忘れて、古い640本・±80度の出力を見ていた）
     - **学び**: 角度の単位はラジアン。`max_angle`は`min_angle`より大きくする（最初は「0、−359」と度で考えていた）
   - ✅ **スキャンを方策の入力に変える仕様を決めた**（2026-09-30）
     - 置き換え: `-inf`（`range_min`より近い）→ **0**、`+inf`（`range_max`以内に何もない）→ **`range_max`（10）**、`NaN`（計測失敗）→ **0**（障害物とみなす安全側）
       - `+inf`を100にしない理由: 意味は「10m以内に何もない」で10と同じ。実測は0〜10なので、100があるとMLPの入力の正規化と固定小数点の精度が崩れる
     - **変換は`policy` crateに置く**（SP1 guestと共有）。blackboxは**生の360本をコミット**し、guestが同じ関数で24本を作る。
       理由: 「実際に動かしたコードそのものを証明する」範囲に入力の作り方まで入り、S3を生データで判定できる。変換がROS側にあると、間引き方や`-inf`の扱いが証明の外に残る
     - **間引きは区間の最小値**（15本ずつ24区間、各区間の最小値を1本にする）。15本に1本を取るだけだと、間の細い障害物（歩行者の脚）を見落とす
   - ✅ **上の仕様を`policy::preprocess_beams(beams_raw: &[f32], range_max: f32) -> Result<[f32; NUM_BEAMS], &'static str>`として実装した**（2026-10-02、`e75ea88`・`557c078`）
     - 区切り方は**正面を区間の中央**にした。`shaped[i] = raw[(i + 353) % 360]`と回してから15本ずつ区切る。`beams[0]`＝`raw[353..360]`＋`raw[0..8]`（右7度〜左7度）
     - 長さが360でなければ`Err`を返す（guestでpanicさせないため。panicすると証明そのものが作れない）
     - 最小値は置き換えのあとに`<`で自分で比べる（`f32::min`は`NaN`を無視するので、置き換え前に使うと`NaN`が消える）
     - `expert()`は置き換えをしない。きれいな24本を受け取る前提（W2でMLPに置き換わっても、入力は同じ関数を通る）
     - **学び**: `for x: T in ...`のようにforのパターンに型注釈は書けない。`&[f32]`から切り出したスライスは`iter_mut()`できない。値を作るなら`let v = if ... { } else { };`の形にする
     - **学び**: 回転の`%`の右側は一周の長さ（360）のまま。ずらす量は左側で決める。`usize`で`i - 7`はアンダーフローするので、`i + 360 - 7`と足し算で書く
     - **学び**: テストの背景値は、確かめたいことが結果に表れる値にする（背景0.0だと、どの区間も最小値が0になり、置いた値が見えない）
   - ✅ 区間の境目のテストを足した（2026-10-02、`b437988`）: `raw[352]`→`beams[23]`、`raw[353]`・`raw[359]`・`raw[0]`・`raw[7]`→`beams[0]`、`raw[8]`→`beams[1]`
   - ✅ `policy_node`を`preprocess_beams()`に切り替えた（2026-10-03、`ad9f7d8`・`b811d5d`。3章を参照）。360本の偽スキャンで`z: 0.8`を確認
     - **決定**: `Err`のときは**停止指令を出す**。gzのdiff-driveは新しい指令が来なければ最後の指令を保つはずで（未確認）、何も出さないと直進し続けうる。
       「入力が壊れていたので停止を選んだ」は記録に残る行動になる（フォレンジックの観点）
     - **決定**: `range_max`は引数にせず`policy::RANGE_MAX`に固定（方策の入力の定義。guestと共有し、学習時の正規化にも使う）
     - **学び**: Rustにはデフォルト引数がない（代わりは定数・`Option`＋`unwrap_or`・関数を分ける）
     - **学び**: `match`は式で、すべての腕が同じ型を返す（`Action`と`None`は混ぜられない）
     - **学び**: `cargo build`はテストをコンパイルしない。シグネチャを変えたら`cargo test`も回す
   - ⬜ **次はここから**（ユーザーが実装）
     1. 仕上げとコミット: ✅ `eprint!`→`eprintln!`、✅ 4本の偽スキャンでErrの経路を確認、✅ `dummy.py`を`scripts/`に移動（いずれも2026-10-03）、✅ コミット（`b811d5d`）
     2. ✅ 片付け（2026-10-03、`a7502a2`）: 定数を`NUM_RANGES_RAW`（360）・`RANGES_PER_BEAM`（15）に整理、ずらし量は`RANGES_PER_BEAM / 2`、`&`と`iter()`に、
        `preprocess_beams`に`///`コメント、`turns_when_blocked`は24本を直接`expert()`に渡す形に戻した。残りは`RANGE_MAX`への`///`（単位m、SDFの`<max>`と揃える）だけ
     3. ノードで`scan.range_max`と`policy::RANGE_MAX`が食い違うときの扱いを決める（警告か`Err`扱いか。SDFだけ変えて定数を直し忘れる事故を防ぐ）
     4. （任意）`dummy.py`で0.3を置く位置を変え、`i=8`（`beams[1]`→旋回）をノード経由で確かめる。✅ `i=30`（`beams[2]`→直進）は確認済み（2026-10-03）
   - ✅ **閉ループで「ずっと旋回する」不具合を直した**（2026-10-03、未コミット）
     - 症状: 直進して箱の手前で止まったあと、回り続ける。スキャンは正面0.211mで、左右対称に`0.211/cos(角度)`、±90度より後ろは`inf`＝**前に約28〜30度傾き、LiDARが床を写していた**（後ろのキャスターが浮いていた）。
       床の線は車と一緒に回るので、`expert()`は永遠に「前方0.5m未満」と判断する
     - 根本原因: 小型化で車輪をx=0（車体中央）に移し、キャスターは後ろ（x=−0.07）だけ。重心はx≈−0.003で**車軸のほぼ真上**。キャスターに約4%しか荷重がなく、止まる・回り始める力で前に倒れる
     - 修正: **前にもキャスター**`caster_front`（x=+0.07、r=0.05、mass 0.05、inertia 0.00005×3、`caster_front_joint` ball）。後ろは`caster_back`／`caster_back_joint`に改名。回転の中心＝LiDARの位置は保った
     - **LiDARの`range_min`を0.08→0.15に変更**: 実機の2D LiDARに近い値。キャスターの先端（0.12）より外側なので、車体に触れる距離のものは必ず`-inf`（→0）になる。
       ただし`-inf`は「0.15より内側に何かある」で、接触そのものではない（衝突の検知はcontactセンサーで取る案が有力）
     - **学び**: 車輪の位置を変えたら、重心と支点（車輪・キャスター）の関係を計算し直す。重心が支点の間に余裕を持って入っているか
     - **学び**: 低い2D LiDARは、傾くと床を障害物として写す。スキャンの形（左右対称の`d/cos`、前半分だけ有限値）から傾きと角度（sin＝高さ/距離）を推定できる
     - **学び**: 1つのmodelの中で、jointの名前も一意にする（重複すると`gz sdf -k`が`joint with name[...] already exists`）
     - **学び**: bridgeのYAMLは、両側で同じ名前なら`topic_name`、違うなら`ros_topic_name`＋`gz_topic_name`。`*_type_name`には型を書く（名前と取り違えやすい）
   - LiDARの残りの仕様: `update_rate`（今10Hz、方策の周期＝反応時間の下限）、トピック名（今`lidar2`。bridgeで`/scan`につなぐ）
3. ✅ `ros_gz_bridge`で`/scan`（← gz `/lidar2`）と`/cmd_vel`（→ gz `/model/vehicle_blue/cmd_vel`）をブリッジし、`policy_node`で走らせた（2026-10-03、`config/bridge.yaml`、未コミット）。
   名前はbridgeのYAMLで合わせる（SDFとノードはそのまま。実機ではbridgeを外すだけ）。LiDARは`lazy: true`、cmd_velは`lazy: false`
   - 未確認: 旋回して箱が前方から外れたあと直進に戻るか、そのまま走り続けた先の振る舞い（ワールドに壁はない）
4. 起動手順を`pixi.toml`の`[tasks]`にまとめる（両PCで同じコマンドで起動できる）。`gz sim -r`（再生状態で起動）を入れる

### SDF作りのメモ

- お手本（`.pixi/envs/default/share/`の下。**直接編集しない**。`pixi install`で上書きされ、gitにも入らない）
  - `gz/gz-sim/worlds/diff_drive.sdf`: 車体・車輪・キャスターの組み方、`gz-sim-diff-drive-system`
  - `gz/gz-sim/worlds/visualize_lidar.sdf`: `gpu_lidar`の書き方と、ワールドに要るシステム（Physics、Sensors＋`ogre2`、SceneBroadcaster）
  - `ros_gz_sim_demos/config/diff_drive.yaml`、`gpu_lidar.yaml`: bridgeのYAMLの書き方
- 構成: 最初はワールドとロボットを`worlds/forensics.sdf`の1ファイルに書く。育ったら`models/robot/model.sdf`に分けて`<include>`する
- **ハマりどころ: ビーム0の向き**（2026-09-30に解決: 360本・`min_angle`=0で`ranges[0]`＝正面にした）。`policy::expert()`（`crates/policy/src/lib.rs:16`）は`beams[0]`が真正面、`beams[1]`と`beams[23]`がその両隣という前提。
  gzのlidarを`min_angle=-π, max_angle=π`にすると`ranges[0]`は真後ろを向く。SDFの角度範囲で合わせるか、`downsample()`で回転させるかはユーザーが決める
  （方策の入力の定義になり、SP1 guestと共有する）。`samples`は24の倍数にし、360度で始点と終点が重複しないようにする
- **SDFの数値はS3の公開パラメータになる**ので、決めたら表にして残す
  - 決まった値（2026-10-03）: LiDAR `range_min` 0.15m／`range_max` 10m（=`policy::RANGE_MAX`）／`update_rate` 10Hz、diff-drive `min_linear_acceleration` −1m/s²・`max_linear_velocity` 0.5m/s、
    車体の前端 0.07m・前キャスターの先端 0.12m（LiDAR中心から）
  - 停止距離の目安: 0.2m/sならv²/2a＝0.02m、反応時間（10Hz→0.1秒）分0.02mを足して約0.04m。障害物が`range_min`（0.15）より内側に入った時点で、止まれるかの境目にいる
  - `max_linear_velocity`、`min_linear_acceleration`（最大減速度） → 停止距離
  - LiDARの`update_rate` → 方策の周期（反応時間の下限）
  - LiDARの最大距離と、障害物が「突然出現する」距離の関係
  - 今の`expert()`（0.2m/sで直進、前方0.5m未満で旋回）が、SDFの加速度制限と矛盾しないか
- **`<pose>`の基準**（SDF 1.x、`relative_to`なし）: `<model>`はワールドから、`<link>`は**モデルのフレームから**測る（jointの親リンクからではない）。
  例: お手本では、モデルz=0.325＋chassis 0.175 → chassisはワールドz=0.5。`lidar_link`の0.5もモデル基準なので、ワールドz=0.825（chassis上面0.784の直上）
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
