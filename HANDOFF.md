# HANDOFF — zk_accident_forensics

最終更新: 2026-10-08
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

## 3. 現在の状態（2026-10-07）

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
- **閉ループ走行を達成**（2026-10-03、`0081853`）: Gazebo → `ros_gz_bridge`（`config/bridge.yaml`）→ `policy_node` → Gazebo。車は箱に向かって直進し、手前で旋回する。
  途中で「旋回し続ける」不具合が出て、原因は**重心が車軸のほぼ真上（3mm後ろ）で、前に倒れていた**こと。前にもキャスターを付けて直した（6章の2）。
  起動手順（ターミナル3〜4つ、プロジェクト直下、この順で。`pixi.toml`の`[tasks]`、2026-10-03）:
  1. `pixi run sim`（GUIなしは`pixi run sim-headless`＝`gz sim -rs`）
  2. `pixi run bridge`
  3. `pixi run policy`（`depends-on = ["build"]`で先にビルドする。**`build`は`policy_node`を含むcrateをビルドすること**。`-p policy`だとライブラリだけで、古いバイナリが動く）
  4. （観察）`pixi run ros2 topic echo /cmd_vel`、`pixi run ros2 topic info /cmd_vel`（Publisher/Subscriptionが各1）
- **ワールドを「8の字」の部屋にした**（2026-10-03、`d1438f0`。箱は削除）。詳細は6章の5
- **`expert()`を作り直した**（2026-10-04、`202797c`）: 窓の最小値によるargmax（greedy）＋前方の距離に応じた滑らかな減速＋塞がったら固定の向きで旋回。8の字の部屋を詰まらずに走る。詳細は6章の5
- **`expert()`を片付けた**（2026-10-04、`dcad969`・`e768bc4`・`ce755e7`）: 窓の最小値の計算を1回に（`min_values`の添字＝`target_beams`の番号）、
  同点は`TIE_EPS`（0.01m）以内なら正面に近い方、`target_beams`を`%`で回して作る（`FRONT_BEAMS_LEN`）、clampを使う、`-inf`/`NaN`の条件をまとめた。`test_eps`を追加し、`cargo test -p policy`は6件通過。詳細は6章の5
- **デモの記録と書き出しができた**（2026-10-04、元のPC、`232d3d2`でコミット済み: `scripts/data_reshape.py`・`pixi.toml`/`pixi.lock`（`mcap`・`pandas`を追加）・`.gitignore`（`/bags`））。
  生の`/scan`と`/cmd_vel`を`ros2 bag record`で`bags/<run>/`に記録し、`data_reshape.py`でCSVにする。`bags/run_01`（71秒、712組）、`bags/run02`（16秒、160組）。詳細は6章の5
  - **bagとCSVはgitに入らない**（`/bags`）。サブPCには無いので、記録し直すかコピーする（4章「サブPCで作業を再開する手順」）
  - 2026-10-05に記録した`bags/test01`（`/scan` 265行、`/cmd_vel` 266行）が手元にある
- **CSVをRustで読めるようになった**（2026-10-05、`5efeb3d`・`b41023d`）: `robot_nodes`のbin `read_data`（`crates/robot_nodes/src/bin/read_data.rs`、`csv`クレートを追加）。
  `pixi run cargo run --bin read_data -- bags/<run名>`（**プロジェクト直下で**実行）で、2つのCSVを`Vec<CmdVel>`と`Vec<Scan>`に読み込む。詳細は6章の5
- **S2の予行に成功**（2026-10-06、`3277236`・`4c9c8e5`）: `bags/test01`の265スキャンすべてで、オフラインで再計算した`preprocess_beams()`→`expert()`の行動が、
  記録された`/cmd_vel`と**ビット単位で一致**（不一致0、対応なし0）。ノードが実際に出した行動を、同じ`policy`のコードをノードの外で動かして再現できた＝guestで同じことをする前提が確かめられた
- **デモの本格的な収集を開始**（2026-10-06）: gzの`/world/diff_drive/set_pose`サービスで開始位置・向きを変えて記録する。`bags/demo_01`（`/scan`・`/cmd_vel`各571件）は
  不一致0・**対応なし4件**。原因は照合の時刻の扱い（`MAX_GAP_NS`が狭い＋`log_time`の順番の入れ替わり）で、ノードの問題ではない。**照合の条件を直した**（2026-10-07、サブPC、未コミット）: ±20ms以内で時刻の差が一番小さいcmdを選ぶ形にし、demo_01は`matched: 571`・test01は`matched: 265`（どちらも対応なし0）。rec01〜14での再確認は元のPCで（6章の5-4）
- **デモの収集を終えた**（2026-10-06夜、**元のPC**）: `bags/rec01`〜`rec14`（14本、`/scan` 2,810件、約6分）。全runをCSVにし、`read_data`で照合して**不一致0**（対応なし33件は取りこぼし。学習には影響なし）。
  run_01・run02は削除済み。**データは元のPCにしかない**（サブPCで学習するならコピーが要る。4章「サブPCで学習を進める手順」）。詳細は6章の5-4
- **`angular`のクラス変換を`policy`に実装した**（2026-10-07〜08）: `class_to_angular(class)`（`0d51896`）と`angular_to_class(angular)`（未コミット）。
  9値の表`ANGLES`（−0.8, −π/4, −π/6, −π/12, 0, π/12, π/6, π/4, 0.8）。`max_idx`→class は `clamp(max_idx−2, 0, 8)`（class 0＝`max_idx` 1・2、class 8＝10・11）。
  `angular_to_class`は`ANGLES`を`==`で探す形（`as usize`の切り捨てを避けた）。`cargo test -p policy`は8件通過
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

### サブPCで作業を再開する手順（2026-10-04時点）

サブPCの初回のセットアップ（pixiの導入など）は済んでいる（6章の0-4）。最後にサブPCで作業したのは9/30ごろで、その後に増えたものを取り込む手順。

0. **前提: 元のPCで、未コミットのもの（`scripts/data_reshape.py`、`pixi.toml`/`pixi.lock`、`.gitignore`、`HANDOFF.md`）をコミットしてpushしておく**
1. 取り込む: プロジェクト直下で`git status`（サブPC側に未コミットの変更がないか）→ `git pull`
2. 環境を更新する: `pixi install --locked`（`mcap`と`pandas`が増えている。lockが書き換わらないこと）
   - 確認: `pixi run python -c "import mcap, pandas, rclpy; print('ok')"`
3. ビルドとテスト: `pixi run cargo build` → `pixi run cargo test -p policy`（6件通過するはず）
   - `target/`はPCごとに作る（RPATHが絶対パスのため）。おかしければ`cargo clean`してからビルド
   - `pixi run`なしの`cargo test`（VS Codeのテストボタンを含む）は`rclrs`のビルドで落ちる
4. 閉ループの確認（ターミナル3つ、3章の起動手順）: `pixi run sim-headless` → `pixi run bridge` → `pixi run policy`。
   ワールドが8の字の部屋になり、車が壁にぶつからずに走り続けることを確かめる（GUIで見るなら`pixi run sim`）
5. **bagはgitに入っていない**（`/bags`）。サブPCで使うデータは次のどちらかで用意する
   - おすすめ: **サブPCで記録し直す**（4の状態で`pixi run ros2 bag record -o bags/<run名> --topics /scan /cmd_vel`）。記録から書き出しまでの流れがサブPCでも動くことの確認にもなる
   - 元のPCのデータを使う: `bags/<run名>/`のフォルダごと（`.mcap`と`metadata.yaml`）をUSBやクラウドドライブなどで運び、`bags/`の下に置く
6. 書き出し: `pixi run python scripts/data_reshape.py bags/<run名>` → 同じフォルダに`data_scan.csv`と`data_cmd_vel.csv`ができる
7. （残っていれば）秘書ノート`~/company`を`git pull`（6章の0-4-7）

注意: サブPCのGazeboはGUIありでRTF70%超（元のPCと同程度）。デモの記録はGUIなし（`sim-headless`）で回すほうが速く、安定する。

### サブPCで学習を進める手順（2026-10-06時点）

**方針（2026-10-06決定）: まずサブPCにあるデータ（`bags/test01` 265件・`bags/demo_01` 571件）で学習の仕組みを作り、調整と本番の学習は全データをそろえてから行う。**
サブPCのデータは少なく偏っている（test01は全速85%）ので、層の大きさ・学習率・回帰か分類かの最終判断や、止まれるかの評価には使わない。
読み込むrunは引数か設定で切り替えられるように（`bags/`の下の複数のrunを読めるように）作る。どちらも同じ`expert()`・同じSDFで記録したので、混ぜて使える。

全データ（`bags/rec01`〜`rec14`）は**元のPCにしかない**（`/bags`はgitの対象外）。サブPCで本番の学習をするなら、上の「作業を再開する手順」の1〜3に加えて:

1. 元のPCで: コードと`HANDOFF.md`をコミットしてpush
2. 元のPCで: データをまとめる。学習に要るのはCSVだけ（約19MB。`.mcap`込みで約28MB）
   - 例: プロジェクト直下で`tar czf ~/rec_csv.tgz bags/rec*/data_*.csv`（`.mcap`も運ぶなら`bags/rec*`）
3. 運ぶ: USBやクラウドドライブなど。WSLからWindows側へは`/mnt/c/Users/<ユーザー名>/...`にコピーすれば見える（サブPCでも同じ）
4. サブPCで: プロジェクト直下で`tar xzf <ファイル>` → `bags/rec01/data_scan.csv`などができる
5. 確認: `pixi run cargo run --bin read_data -- bags/rec01`が元のPCと同じ結果（`matched: 600`、`no_action: 2`）になる
- 学習の計算量: MLP（24→数十→2）と約3,000件なら、**CPUで数秒〜数十秒**。サブPC（Ryzen 5 5625U、RAM約12GB、GPUはAMD内蔵のみ）で十分。BurnはCPUのバックエンド（`ndarray`）で始める
- **どちらのPCを学習の正本にするか決める**: 学習済みの重みは`policy`に埋め込んでgitに入るので、PCをまたいでも困らない。データを足すとき（記録はGazeboが要る）は、記録したPCから再びコピーする

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

## 6. 次にやること（W2。次は5-5「学習の設計」。デモの収集は10/6に終了）

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
   - ✅ **閉ループで「ずっと旋回する」不具合を直した**（2026-10-03、`0081853`）
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
3. ✅ `ros_gz_bridge`で`/scan`（← gz `/lidar2`）と`/cmd_vel`（→ gz `/model/vehicle_blue/cmd_vel`）をブリッジし、`policy_node`で走らせた（2026-10-03、`config/bridge.yaml`、`0081853`）。
   名前はbridgeのYAMLで合わせる（SDFとノードはそのまま。実機ではbridgeを外すだけ）。LiDARは`lazy: true`、cmd_velは`lazy: false`
   - `0081853`には`config/diff.yaml`も入っている（中身と要否は未確認。お手本のコピーなら消してよい）
4. ✅ 起動手順を`pixi.toml`の`[tasks]`にまとめた（2026-10-03、`5590285`）: `build`（`cargo build`）、`sim`（`gz sim -r`）、`sim-headless`（`gz sim -rs`）、`bridge`、`policy`（`depends-on = ["build"]`）
   - **学び**: taskの中に`pixi run`は書かない（taskは最初からpixi環境の中で実行される）
   - **学び**: `depends-on`は前のtaskの終了を待つ。終わらないプロセス（gz、bridge、ノード）どうしはつなげない。まとめて起動するならROS 2のlaunchファイル（`policy_node`はcolconのパッケージでないので、実行ファイルのパスを直接指定する書き方を調べる）
   - **学び**: `cargo build -p policy`はライブラリだけをビルドし、`policy_node`は作り直されない
5. **（作業中）W2の準備: ワールドとエキスパートとデモの記録**（2026-10-03に方針を決定）
   - ✅ **ワールド**（`d1438f0`）: 外周4m×4m（内側x −1.9〜1.9、y 0.1〜3.9、壁は厚さ0.2・高さ1.0）。内側の仕切り（厚さ0.1・高さ0.5）:
     wall_1（x=1、y 1〜3）、wall_2（x=−1、y 1〜3）、wall_3（x=0、y 3〜4、上の壁につながる）、wall_4（x=0、y 0〜1、下の壁につながる）。
     **左右2つのループが中央（x=0、y 1〜3）でつながる「8の字」**。車は(0, 2)、+x向きで開始。前方のwall_1まで0.9m
     - ねらい: 両回りのループで左右どちらにも曲がるデモが集まる。中央の交差点は左右の判断の場所。wall_3/wall_4の陰はシナリオBの死角（飛び出し）の候補
     - **学び**: **`gpu_lidar`はvisualを描画して測り、物理はcollisionで計算する**。visualとcollisionの寸法が違うと「見えるのに当たらない／当たるのに見えない」場所ができ、S3の判定が成り立たない。必ず同じ寸法にする
     - **学び**: 隙間は車幅（車輪の球まで含めて0.28m）と比べる。0.4mの隙間は通れても、前方の判定（±22度）が両側の壁を拾って入れない
   - ✅ **`expert()`を作り直した**（2026-10-03〜04、`845fad2`・`202797c`）。8の字の部屋をぶつからず、詰まらずに走り続ける。`cargo test -p policy`は5件すべて通過
     - 仕様:
       1. 候補は前半分（右90度〜左90度の13区間、`target_beams`に右→左の順で並べ直す。番号`i`の角度＝`−π/2 + i·π/12`）
       2. **方向の選び方＝窓の最小値によるargmax**: 各方向について「その区間と両隣の3区間の最小値」をスコアにし（＝車幅分の隙間があるか）、スコア最大の窓の中心を選ぶ。
          同点は「正面から外へ、左を先に」の順（`index_array = [6,7,5,8,4,…]`）で最初のもの。両端（0と12）は窓が作れないので候補外
       3. `angular`＝選んだ角度を`±W_MAX`（0.8）で切る（比例制御。目標が正面に来るほど弱まる）
       4. **速度は正面5区間（±37.5度）の最小値`front_min`で決める**: `D_SLOW`（0.5）以上で`VMAX`（0.2）、その下は`VMAX·(front_min − D_TARGET)/(D_SLOW − D_TARGET)`、`D_TARGET = D_STOP − 0.05`
       5. **`front_min < D_STOP`（0.25）なら、argmaxを使わず、常に左に`W_MAX`でその場旋回**（空いている側に関係なく固定）
     - 経緯と学び（どれもレポートの工夫点・S3の材料になる）:
       - 速度を「一番空いている方向の距離」で決めると、正面の壁に全速で突っ込む。**向き（argmax＝最大値）と速さ（危険度＝最小値）は別の基準**
       - 最小値のループで初期値を`INFINITY`にしたのに比較の向きが最大値のまま→一度も更新されず常に全速。**初期値と比較の向きはセットで変える**（テストが捕まえていた）
       - 1区間のargmaxだと「正面1区間だけ空き、両隣が近い」で向いたのに進めず固まる。**方向を選ぶ基準を、進む基準（車幅）にそろえる**→窓の最小値
       - 窓のスコアの最大値を、生の1区間の値と比べていた→窓の中心でなく端を選んでいた（テストの期待値と偶然一致）
       - **速度を残りの距離に比例させると、目標に漸近して永遠に届かない**（1ステップで残りの8%しか縮まない。`/cmd_vel`の`linear`が2.6e−5→9.3e−6と減り続けた）。目標を`D_TARGET`に下げて有限時間で`D_STOP`を越えさせる
       - **往復（2種類）**: 塞がったときの固定の旋回の向きとargmaxの向きが食い違うと、「argmaxへゆっくり→固定の向きへ急旋回」を繰り返す。
         しきい値の境目でも同じことが起きた。**塞がっている間は向きを決める規則を1つだけにする**（argmaxを使わない）ことで原理的に解消
       - **`expert()`に状態（直前の旋回の向きなど）を持たせない**: MLPは今の24本だけを見るので、同じ入力に違う出力のデモが混ざると真似できない。模倣学習のお手本は**beamsだけで決まる純粋な関数**にする
       - テストは「直す前のコードで落ち、直した後で通る」入力にする（往復のテストは左を3区間空ける。1区間だけだと窓のスコアが0で、古いコードでも通ってしまう）
       - `pixi run`なしの`cargo test`（やVS Codeのテストボタン）は`ROS_DISTRO`がなく`rclrs`のビルドで落ちる。`pixi run cargo test -p policy`で実行する
     - ✅ **片付け**（2026-10-04、`ce755e7`）: 重複の解消、`TIE_EPS`（差が0.01m未満なら同点→`index_array`の順）、`target_beams`をループで作る、clippyの3件（clamp、`-inf`/`NaN`の`if`、スライスのコピー）
       - **学び**: 配列の長さは`const`でないと書けない（`let`の変数はE0435）。長さは定数から決め、同じ値を2か所に書かない
       - **学び**: 窓のスコアは最小値なので、「ほぼ同点」のテストは**その窓だけに入っている区間を一番小さく**する。テストは比較を`==`に戻して**落ちる**ことを確かめる（`test_eps`は確認済み）
       - **学び**: `4..8`は8を含まない。書き換えで`front_min`が4区間（右30〜左15度）になりかけた。今は`4..=8`。テストでは捕まらなかった
       - 右端の窓（1番）だけが空いている入力でパニックしないことは、scratchpadで全窓について確認済み（テストは追加しないと決めた）
       - 窓の幅3区間は維持（5区間にすると通路で両側の壁を拾い、候補も±60度に狭まる）。正面の6・`index_array`・`front_min`の`4..=8`は手書きのまま（ビーム数を変えるときにまとめて定数化）
     - 残り（動作に影響なし、ついでに直す程度）: 単位（m・m/s）と`FRONT_BEAMS_LEN`のコメント、72行目の`‐`、clippyの2件（`for i in 0..FRONT_BEAMS_LEN`は残す方針、`for i in 4..=8`はスライス`&target_beams[4..=8]`を回せば消える）、（任意）左30度だけ`D_STOP`未満で停止するテスト
     - 未確認: 同じ軌道を繰り返すか（デモの多様性）。旋回方向を固定したので、左に道があっても最悪ほぼ1周回る
   - ✅ **デモの記録**（2026-10-04、元のPC）
     - **決定: 生の`/scan`と`/cmd_vel`をそのまま記録する**（ノードは変えない）。教師ラベル（行動）は記録した`/scan`から`preprocess_beams()`→`expert()`で**オフラインで再計算**する
       - 理由: `expert()`はbeamsだけで決まる純粋な関数なので、ラベルは何度でも作り直せる。前処理や`expert()`を変えても集め直さずに済む。blackboxが生の360本をコミットする設計とも同じ形
       - 記録した`/cmd_vel`は、再計算した行動と一致するかの確認に使う（**S2の予行**。食い違えばノードとオフラインの計算がずれている）
     - 記録（3つ起動してから、4つ目のターミナルで）: `pixi run ros2 bag record -o bags/<run名> --topics /scan /cmd_vel`、`Ctrl+C`で止める。確認は`pixi run ros2 bag info bags/<run名>`
       - `--topics`（複数形）で、トピックは空白区切り（`[ ]`やカンマは不可）。保存形式の既定は`mcap`
       - `-o`は1回ごとに別のサブフォルダにする（既にあるフォルダを指定するとエラー）。`-o bags`のように直下に作ると、次回から使えない
       - bridgeのLiDARは`lazy: true`なので、`policy_node`を起動してから記録する
     - 結果: `/scan`と`/cmd_vel`は同数（1スキャンにつき1指令）、10Hzで取りこぼしなし。約30KB/秒（10分で約18MB）
     - **書き出し**: `pixi run python scripts/data_reshape.py bags/<run名>` → 同じフォルダに`data_scan.csv`と`data_cmd_vel.csv`
       - `data_scan.csv`: `header_stamp`（シミュレーション時刻、ns）、`log_time`（bagの受信時刻、ns）、`r0`〜`r359`。run02は160行×362列、`header_stamp`はすべて0.1秒刻み（**シミュレーション時刻で正確に10Hz**）
       - `data_cmd_vel.csv`: `log_time`、`linear_x/y/z`、`angular_x/y/z`。`Twist`にはheaderがないので時刻は受信時刻だけ
       - `inf`/`nan`は**Pythonで置き換えない**（置き換えはRustの`preprocess_beams()`の仕事。Rustの`"inf".parse::<f32>()`は読める）。部屋が4m四方なので、今のデータに`inf`は0件
       - 読み出しは`mcap`（`make_reader`→`iter_messages()`）＋`rclpy.serialization.deserialize_message`。`rosbag2_py`も環境にある
       - **学び**: `iter_messages()`の`message`はmcapの記録（`log_time`と`data`）で、`header`などは`deserialize_message`で戻したメッセージ側にある。`header.stamp`は`sec`/`nanosec`のオブジェクトで、そのままCSVにすると文字列になる
       - **学び**: `open()`は`*`を展開しない（`Path(...).glob()`を使い、`list()`にしてから`[0]`）。Pythonの長さは`len(x)`。引数の先頭に`/`を付けると絶対パスになる
     - 気づき（学習の設計で使う）: `angular_z`は0.0／0.262／0.524／0.785／0.8／−0.8の**飛び飛びの値**（13方向のargmax＋`W_MAX`で切るため）。MLPを回帰にするか13方向の分類にするかを決める
     - 確かめておくこと: bagの`Duration`は実時間。GUIありはRTF約70%なので実時間では約7Hzになるはず。S3で反応時間や停止距離を判定するときは`header.stamp`（シミュレーション時刻）を使う。RTFは`gz topic -e -t /stats`の`real_time_factor`
   - ⬜ **次はここから: Rustでラベルを作る＋S2の予行**
     1. ✅ 未コミットの4つと`HANDOFF.md`をコミットしてpush（`232d3d2`）
     2. ✅ 置き場所は**`robot_nodes`のbin `read_data`**にした（2026-10-05）。`policy`は`no_std`なので入れない。Burnの学習用crateは別に作る
     3. ✅ 2つのCSVを読み、各スキャンを`preprocess_beams()`→`expert()`に通し、`log_time`で「スキャンの直後に来た`/cmd_vel`」と対応させ、一致するか比べる
        - ✅ 読み込み（2026-10-05、`b41023d`）: `CmdVel { time: u64, linear_x: f64, angular_z: f64 }`（0・1・6列目）、`Scan { time_stamp: u64, log_time: u64, ranges: Vec<f32> }`（`record.iter().skip(2).map(parse::<f32>)`で360本）
        - ✅ 各`Scan`の`ranges`を`preprocess_beams(&scan.ranges)`→`expert()`に通す（2026-10-06）
        - ✅ `log_time`で直後の`CmdVel`を探して比べる（2026-10-06）: `cmd_vels.iter().find(|c| c.time > scan.log_time && c.time - scan.log_time < MAX_GAP_NS)`、
          `action.linear as f64 != cmd.linear_x || action.angular as f64 != cmd.angular_z`で不一致を数える。**test01で不一致0・対応なし0**
          - `MAX_GAP_NS`は10ms。実測でscan→cmd_velの差は0.2〜0.8ms（中央値0.5ms）。スキャンの間隔は実時間で最短35ms・中央値100ms・最長136msと揺れるので、100msだと指令が欠けたときに次のスキャンへの指令を拾いうる
          - **10msは狭すぎた**（demo_01で判明、5-4を参照）。「直後」ではなく「±20ms以内で一番近い」に直す
          - **添字（`scan[i]`と`cmd_vel[i]`）で対応させない**。test01は最初の`/cmd_vel`が最初の`/scan`より約100ms早い（記録開始前のスキャンへの指令）ので、`scan[0]`↔`cmd_vel[1]`。直進が続く間は1つずれても値が同じで、間違いに気づけない
        - ✅ 行数が1つ違う（`test01`は`/cmd_vel`が266行、`/scan`が265行）のは先頭の余分な`/cmd_vel`。時刻で対応させるので、余った`/cmd_vel`は自然に使われない。scan側で対応が無いものは`no_action_count`で数える
        - ✅ 仕上げ（2026-10-06、`4c9c8e5`）: `total length / matched / unmatched / no_action`を表示、不一致は`eprintln!`で`log_time`・両方の値・時刻差を表示し、1件でもあれば`exit(1)`。
          `CmdVel.time`→`log_time`に改名。**値をわざとずらして不一致を検出できることを確認済み**（シナリオCの予行）
          - 残り（任意）: 最後の`eprint!`→`eprintln!`、`cargo fmt`（長い行を足したあと未適用）、フィールドの省略記法
        - **学び**: binのファイルでは`crate::`はそのbin自身を指す。依存している別のcrateは`use policy::...`のようにcrate名で使う（`mod`は自分のcrateにファイルを取り込む宣言）
        - **学び**: ワークスペースの直下での`cargo add`は`-p robot_nodes`で追加先を指定する
        - **学び**: `Path::join`に`/`で始まる文字列を渡すと絶対パスとして扱われ、元のパスが捨てられる（Pythonと同じ落とし穴）。相対パスの引数は、起動したディレクトリを基準に解決される
        - **学び**: `log_time`（19桁のns）は`u64`で読む（`f32`だと桁が落ちる）。`Twist`の中身は`f64`
        - **学び**: `for`の中に置いた`println!("{:?}", v.last())`は、毎回「今pushした要素」を表示する（`}`の位置に注意。`cargo fmt`で構造を見える形にする）
        - **学び**: 時刻はすべてns。`log_time`は実時間（UNIX時刻）、`header_stamp`はシミュレーション時刻。scanとcmd_velは`log_time`どうしで比べる
        - **学び**: 比べるときは**狭い型を広げる**（`f32`→`f64`は値が変わらない。`f64`→`f32`は丸めで差が消えうる）。記録側（`Twist`の`f64`＝実際に送った指令）を丸めず、`expert()`側を`as f64`で広げる＝ノードと同じ変換
        - **学び**: `a > b && a - b < GAP`は、`&&`が左で止まるので`u64`の引き算のアンダーフローを避けられる
        - **学び**: `log_time`は**bagの記録プロセスが各トピックを受け取った時刻**で、ノードが送った順ではない。`/scan`と`/cmd_vel`は別々に受け取るので、ほぼ同時だと順番が入れ替わる（demo_01で0.035ms逆転）。「だいたいの時刻」には使えるが、因果の順番の証拠にはならない
     4. ✅ デモを本格的に集める（2026-10-06、元のPC）。結果は下の「rec01〜rec14の結果」
        - 狙い: test01は`angular_z`=0が48%（126/265）、`linear_x`=0.2（全速）が85%（225/265）で、旋回・減速の場面が少ない。**旋回・減速を増やす**ため、短いrun（1〜2分）を開始位置・向きを変えて8〜10本（計10分前後、約6000件）。壁に向いた位置や角の近くからも始める
        - 置き直し: Gazebo起動中に
          `pixi run gz service -s /world/diff_drive/set_pose --reqtype gz.msgs.Pose --reptype gz.msgs.Boolean --timeout 1000 --req 'name: "vehicle_blue", position: {x: 1.45, y: 2.0, z: 0.0}, orientation: {z: 0.7071, w: 0.7071}'`
          - ワールドに`gz-sim-user-commands-system`があるので使える。ワールド名はSDFの`<world name="diff_drive">`。`pixi run gz service -l | grep set_pose`で確認（**Gazeboが起動していないと何も出ない**）
          - 向きはクォータニオン: yaw θなら`z = sin(θ/2)`、`w = cos(θ/2)`（+x: `{w: 1}`、+y: `{z: 0.7071, w: 0.7071}`、−x: `{z: 1, w: 0}`）
        - 開始位置の候補: 中央(0, 2)、右の通路(1.45, 2)、左の通路(−1.45, 2)、下の通路(±0.5, 0.55)、上の通路(±0.5, 3.45)、壁の手前(0.6, 2)+x向き（wall_1まで0.4m、減速用）
        - 手順: 置き直す → `pixi run ros2 bag record -o bags/<run名> --topics /scan /cmd_vel` → 1〜2分で`Ctrl+C` →
          **`pixi run ros2 bag info bags/<run名>`で`/scan`と`/cmd_vel`が両方ほぼ同数あることを確認** → `data_reshape.py` → `read_data`
        - 記録した開始位置（run名と一緒にここに追記する）:

          | run | 開始位置 (x, y) | 向き | 長さ | 件数 | 照合 |
          |---|---|---|---|---|---|
          | demo_01 | （要記入） | （要記入） | 約90秒 | 571 | 一致567・不一致0・対応なし4 |

        - **demo_01の対応なし4件の正体**（2026-10-06に調査）
          - 3件（scan 178・292・501）: 直後のcmdまで10.1〜11.2ms。`MAX_GAP_NS`（10ms）をわずかに超えただけ（直前のcmdは125ms以上前で取り違えの心配なし）。差の中央値は0.57msだが、負荷で5〜11msまで延びる
          - 1件（scan 81）: **cmdがscanより0.035ms早く記録された**。直後のcmdは次のスキャンへの応答（127ms後）で、10msの制限で正しく除外された
          - demo_01は最初のcmdが最初のscanの0.66ms後（`scan[0]`↔`cmd[0]`）。test01（`cmd[1]`）と違い、添字で対応させない判断の裏付け
        - ✅ **照合の条件を直した**（2026-10-07、サブPC）: 「後で最初のcmd」→「**時刻の差の絶対値が一番小さいcmdを±20ms以内から選ぶ**」。
          `cmd_vels.iter().min_by_key(|c| c.log_time.abs_diff(scan.log_time)).filter(|c| c.log_time.abs_diff(scan.log_time) <= MAX_GAP_NS)`。表示の時刻差も`abs_diff`に（cmdが先だと`u64`の引き算でpanicするため）。
          **demo_01は`matched: 571`、test01は`matched: 265`（対応なし0）**。20msの理由: 実測の差は最大約11ms、スキャンの間隔は最短35ms（前のスキャンへのcmdは約34ms以上前）なので、窓に入るのは正しい組だけ
          - 残り: `MAX_GAP_NS`のコメント（10msのまま）、`cargo fmt`、コミット。元のPCでrec01〜14を再照合し、対応なし33件がどこまで減るかを下の表に追記する
          - **学び**: `abs_diff`は`u64`のメソッドで`use`は要らない（`a.abs_diff(b)`）。`find`は条件に合う最初の1件、`min_by_key`は全件から最小を選ぶ。`Option::filter`で選んだあとに窓を確かめる
          - **学び**: `log_time`の前後には意味がない（bagの受信順。因果は必ずscan→cmd）。条件に`c.log_time > scan.log_time`を残すと、順番が入れ替わった組（scan 81）を窓をいくら広げても拾えない
        - **W3への要件（blackbox）**: 時刻から組を推測するのは根本の弱点。S1/S2は「この観測→この行動」の組をコミットするので、**組はノードの中で確定させる**
          （例: `policy_node`が行動に元のスキャンの`header.stamp`を付けて記録する）。`blackbox_node`の設計で決める
        - **学び**: `ros2 bag record`は存在しないトピック名を指定してもエラーにならず、現れるのを待ち続ける（demo_01の1回目は`/cmd_vel`しか入らず、`data_scan.csv`が1バイトになった）。記録後に`ros2 bag info`で確かめる
        - **学び**: `Path(...).glob()`は、フォルダが無くてもエラーにならず空を返す（`demo01`と`demo_01`の打ち間違いで「fileが1個ではありません」）。`is_dir()`で先に確かめるか、メッセージに件数を入れる
        - **軌道は繰り返す**（確認済み）: シミュレーションも`expert()`も決まった計算なので、走り続けると同じ周回に落ち着く。置き直すと**逆回りの周回**に入ることもある（左右両方の曲がり方が集まるので良い）。
          新しいデータは「置き直してから周回に合流するまで」と「各周回の1周ぶん」。長く回しても1周を超えた分は繰り返し → 短いrunを本数多く
        - 記録は`pixi run ros2 bag record -o bags/recNN --topics /scan /cmd_vel`（`record`を忘れない）。時間で止めるなら先頭に`timeout -s INT <秒>`
        - **rec01〜rec14の結果**（2026-10-06）

          | run | 秒 | `/scan` | `/cmd_vel` | 一致 | 対応なし |
          |---|---|---|---|---|---|
          | rec01 | 85 | 602 | 602 | 600 | 2 |
          | rec02 | 92 | 647 | 647 | 644 | 3 |
          | rec03 | 18 | 129 | 129 | 127 | 2 |
          | rec04 | 15 | 102 | 101 | 101 | 1 |
          | rec05 | 16 | 111 | 105 | 105 | 6 |
          | rec06 | 21 | 152 | 143 | 143 | 9 |
          | rec07 | 12 | 87 | 80 | 79 | 8 |
          | rec08 | 22 | 153 | 153 | 153 | 0 |
          | rec09 | 12 | 80 | 89 | 80 | 0 |
          | rec10 | 27 | 192 | 192 | 192 | 0 |
          | rec11 | 13 | 92 | 92 | 92 | 0 |
          | rec12 | 14 | 100 | 100 | 98 | 2 |
          | rec13 | 30 | 204 | 218 | 204 | 0 |
          | rec14 | 22 | 159 | 159 | 159 | 0 |

          - **不一致は全runで0**。対応なしは計33件（このときの照合の条件は10ms。10/7に±20msの最近傍に直したので、元のPCで再照合する）
          - rec05〜07・09・13は`/scan`と`/cmd_vel`の数が5〜14件ずれる（途中の取りこぼし。周期は実時間で約7Hz＝GUIありで負荷が高かった可能性）。**学習には残す**と決めた
            （ラベルは`/scan`から再計算するので`/cmd_vel`の欠けは無害。短いrun＝置き直しの過渡で、多様さの元）。次に集めるなら`sim-headless`で
          - **W3への材料**: 記録が取りこぼすと、blackboxのハッシュチェーンが途切れる。取りこぼしの扱い（欠番をどうコミットするか）を設計で決める
        - **行動の分布**（全14本、記録された`/cmd_vel`）: `angular_z`は直進43.6%・左33.3%（うち`W_MAX`の8.8%は塞がったときの固定の左旋回を含む）・右23.1%。
          `linear_x`は**全速87.9%・減速8.8%・停止3.4%**
          - **減速・停止が少ない**。回帰だと「いつも0.2」で誤差が小さくなり、止まることを覚えないおそれ（S3で一番大事な振る舞い）。
            まず今のデータで学習し、壁の手前で止まれるかを見る。だめなら、壁の手前や角に向けたrunを足す／減速・停止をオーバーサンプリングする／損失に重みを付ける
     5. **（作業中）** 学習用の出力の形を決める: 先に「回帰か13方向の分類か」を決め、`read_data`から「24本のbeams＋`expert()`の行動」のCSVを書き出すか、Burnのcrateで直接読むかを選ぶ
        - **決定（2026-10-07）: 回帰と分類の組み合わせにする**（`angular`は分類、`linear`は回帰を含む形）
        - 事実: `angular`の候補は`max_idx`1〜11の**11方向**（13方向ではない）で、±`W_MAX`で切るため出る値は**9種類**（−0.8, −0.785, −0.524, −0.262, 0, 0.262, 0.524, 0.785, 0.8）。
          `linear`は減速区間（`front_min` 0.25〜0.5m）で連続（0.033〜0.2）、0.25m未満で0にとび、そのとき`angular`は0.8に固定
        - 回帰だけだと、argmaxの不連続な境目で平均（正面）に逃げる・全速88%に引っぱられて止まり切らないおそれ。分類だけだと減速の連続値を表せない
        - **決定（2026-10-07）: 頭を3つ**＝`angular` 9クラス／モード3クラス（全速・減速・停止）／減速の量の回帰（減速のサンプルだけ損失）。推論はモードで組み立て、停止なら`(0, 0.8)`に固定（ありえない組み合わせを出力の形で防ぐ）。
          損失は`CE(angular)+CE(モード)+λ·MSE`、モードに重み（停止3.4%）。評価は停止クラスの再現率
        - 提案中（未決）: ラベルは記録の`/cmd_vel`ではなく`preprocess_beams()`→`expert()`の再計算から作る。`read_data`が「24本＋ラベル」の中間CSVを書き出し、Burnのcrate（ROS非依存）はそれを読む。正規化`beams / RANGE_MAX`は`policy`に置く
        - **決定（2026-10-07）: `angular`は9クラス**（0.785と0.8はまとめない。expertの値をそのまま再現できる）
        - ✅ `angular`↔クラスの変換（`class_to_angular`・`angular_to_class`）を実装（2026-10-08）。学び: `angular`は向きではなく**角速度**（選んだ向きの角度を`±W_MAX`で切って使う）。クラスは多対一（`max_idx` 1・2→class 0、10・11→class 8）
        - ⬜ **次はここから**
          1. `angular_to_class`の整理: 先頭の±0.8の`if`と範囲チェックは`ANGLES`の探索に含まれるので不要（`NaN`も`Err`になる）。
             **テストを足す**: `expert()`が実際に出す`angular`（`max_idx` 1〜11）で`angular_to_class`が`Ok`になり、`class_to_angular`で元に戻ること（`==`比較の前提の確認）
          2. **モード判定**（全速/減速/停止＝`front_min`と`D_SLOW`/`D_STOP`）を`expert()`と共通の関数にする。停止のとき`angular`は0.8固定でclass 8と衝突するので、モードを先に決め、停止サンプルは`angular`の損失を掛けない
          3. `read_data`に中間CSV（24本のbeams＋`angular`のクラス番号＋モード＋`linear`）の書き出しを足す（ラベルは`preprocess_beams()`→`expert()`の再計算から。まず`bags/demo_01`・`test01`で確認 → クラス分布を出力）
          4. Burnの学習用crateを作る
   - 同じ軌道の繰り返しを避ける工夫（開始位置・向きを変える。gzのサービスで車を置き直すなど）は、デモ収集の段階で検討

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
- **整数のオーバーフローはdebugとreleaseで振る舞いが違う**（debugはpanic、releaseは黙ってwrap）。SP1 guestは常にreleaseでビルドされる。
  W2で固定小数点を書くときは、`policy`の中で`checked_*`/`saturating_*`/`wrapping_*`を使い、振る舞いを明示する（ノードとguestで結果を一致させるため）。
  `[profile.release] overflow-checks = true`も選択肢。ノードは当面debugで動かす（処理が軽いため。2026-10-03決定）

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
