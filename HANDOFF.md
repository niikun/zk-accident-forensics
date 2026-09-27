# HANDOFF — zk_accident_forensics

最終更新: 2026-09-28
このリポジトリで作業を始めるセッション向けの引き継ぎ。まずこのファイルを読むこと。

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

### 動画シナリオ（約3分）

問題提起 → 通常走行（A） → 飛び出しで衝突し「不可避」を証明（B） → ログ改ざん・モデルのすり替えを検証が弾く（C、赤×） → 仕組み図 → 応用。
余力があれば、バグのある旧モデルで衝突し「AI過失」が証明される場面（B'）を入れる。

## 3. 現在の状態（2026-09-28）

- 初回コミット済み: `crates/policy`（no_std、仮のルールベース`expert()`）と`crates/robot_nodes`（`policy_node`）
- `policy_node`: `/scan`を24本に間引いて方策に渡し、`/cmd_vel`（`geometry_msgs/Twist`）に出す
- 疎通確認済み: `ros2 topic pub`で前方0.3mの偽スキャンを流すと`angular.z=0.8`（停止・旋回）が返る
- `cargo test -p policy`はテスト1件（`turns_when_blocked`）

## 4. 環境の事実（ハマりどころ）

- OS: WSL2（Ubuntu）、ROS 2 **Lyrical**（`/opt/ros/lyrical`）、rustc/cargo 1.97
- マシン: 12コア、RAM 15GB、GPU Quadro T1000
- **`rclrs`は0.8以上を使う**。0.7はLyrical非対応で、`Unsupported ROS distribution`というコンパイルエラーになる
- **`ros-env`は0.3**（rclrs 0.8と揃える）。メッセージは`use ros_env::*;`で`sensor_msgs::msg::LaserScan`などが使える
- Lyricalは主要メッセージのRustバインディングを`/opt/ros/lyrical/share/<pkg>/rust`に同梱している。
  **colconもrosidl_rustのcloneも不要**。`source /opt/ros/lyrical/setup.bash`のあとに`cargo build`で通る
- `robot_nodes/package.xml`は将来colconで扱う場合に備えて置いてある（現状は使っていない）
- **Gazebo（ros_gz）は未導入**。導入には`sudo apt install ros-lyrical-ros-gz`が必要で、ユーザーに実行してもらう
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

1. ユーザーにros_gzをインストールしてもらう（`! sudo apt install ros-lyrical-ros-gz`）
2. 差動二輪＋LiDARのロボット（TurtleBot3相当、SDFを自作でも可）と障害物のあるワールドを`worlds/`に置く
3. `ros_gz_bridge`で`/scan`と`/cmd_vel`をブリッジし、`policy_node`で走らせる
4. 起動手順をREADMEかスクリプトにまとめる

## 7. 設計方針・予定のcrate

- `policy`: `no_std`でROSに依存させない（SP1 guestから使うため）。学習後は重みを埋め込んだ固定小数点のMLPに置き換える。`expert()`はデモ収集用として残す
- `robot_nodes`: ROS2ノード群。`policy_node`、今後`blackbox_node`や障害物（歩行者）を動かすノードを追加
- 今後追加予定: 学習用crate（Burn）、SP1一式（`program/`=guest、`script/`=host、`lib/`=共有型）。
  SP1は`cargo prove new --bare`のテンプレートの流儀に合わせる（参考: `~/project/wood_zk_traceability`、`~/project/mpc_group_purchasing`）
- 証明は**事後にオフラインで作る**（commit-now-prove-later）。走行中はコミットだけ行う

## 8. リスクと保険

- Gazeboが重い・WSLgで表示できない場合 → Rust製の2Dシミュレータ＋rerunで可視化に切り替える（課題は他ツール可）
- RAM 15GBなので、SP1はcore/compressed証明まで。証明する区間は事故前後の数十ステップに絞る。Groth16ラップやオンチェーン検証はやらない
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
