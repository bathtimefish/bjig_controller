# サンプルコード

このディレクトリには `bjig_controller` ライブラリの使用方法を示すサンプルプログラムが含まれています。

## 前提条件

サンプルを実行する前に、以下が必要です:

1. **BraveJIG USBルーター**がコンピュータに接続されていること
2. **bjig CLI**がインストールされており、PATHに含まれているか環境変数で指定されていること
3. **環境変数**が設定されていること(以下を参照)

## 環境変数

サンプルを実行する前に、以下の環境変数を設定してください:

```bash
# 必須: BraveJIGルーターが接続されているシリアルポート
export BJIG_CLI_PORT=/dev/ttyACM0  # Linux/macOS
# または
# set BJIG_CLI_PORT=COM3            # Windows

# 必須: ボーレート(通常は115200)
export BJIG_CLI_BAUD=115200

# オプション: bjig CLIバイナリのパス(PATHに含まれていない場合)
export BJIG_CLI_BIN_PATH=/path/to/bjig

# オプション: モジュール設定ファイル
export BJIG_CLI_MODULE_CONFIG=./module-config.yml
```

## サンプルの実行方法

各サンプルは `cargo run --example <サンプル名>` で実行できます:

```bash
cargo run --example simple_router
```

`RUST_LOG` 環境変数を設定すると、詳細なログを出力できます:

```bash
RUST_LOG=debug cargo run --example simple_router
```

## 利用可能なサンプル

### 1. `simple_router.rs`

ファームウェアバージョンの取得、スキャンモード、モジュールIDなど、基本的なルーター操作を示します。

**デモ内容:**
- 環境変数からコントローラーを作成
- ルーターバージョンの取得
- スキャンモードの取得
- 接続されているモジュールの一覧表示

**実行方法:**
```bash
cargo run --example simple_router
```

### 2. `env_config.rs`

環境変数を使用してコントローラーを設定する方法を示します。

**デモ内容:**
- 環境変数から設定を読み込む
- 現在の設定を表示
- 簡単なコマンドで設定をテスト

**実行方法:**
```bash
cargo run --example env_config
```

### 3. `restart_router.rs`

適切なタイミングと検証を含む、完全なルーター再起動シーケンスを示します。

**デモ内容:**
- ルーターの停止
- 適切なシャットダウンのための待機
- ルーターの起動
- ルーターが動作していることの確認

**実行方法:**
```bash
cargo run --example restart_router
```

**注意:** このサンプルは必要な待機時間のため、完了までに約11秒かかります。

### 4. `module_control.rs`

インスタントアップリンクやパラメータ管理など、モジュール固有の操作を示します。

**デモ内容:**
- センサーからインスタントアップリンクデータを取得
- モジュールパラメータの読み取り
- センサーIDとモジュールIDの使用

**実行方法:**
```bash
# コード内のsensor_idとmodule_idを実際のデバイスに合わせて編集してください
cargo run --example module_control
```

**注意:** コード内の `sensor_id` と `module_id` 変数を実際のデバイスに合わせて修正する必要があります。

### 5. `monitor.rs`

コールバック処理を使用したアップリンクデータのリアルタイム監視を示します。

**デモ内容:**
- モニタープロセスの開始
- JSONアップリンクデータの1行ずつの処理
- コールバックを使用したデータ処理
- 受信データの収集と表示
- 条件が満たされた後のモニター停止(5項目収集)

**実行方法:**
```bash
cargo run --example monitor
```

**期待される動作:** このサンプルは5つのJSONアップリンクメッセージを収集し、その後自動的に停止します。

### 6. `monitor_with_ttl.rs`

タイムアウト(TTL - Time To Live)を使用したリアルタイム監視を示します。

**デモ内容:**
- タイムアウト付きでモニタープロセスを開始
- 時間制限付きでJSONアップリンクデータを処理
- TTL付きコールバックの使用
- 以下のいずれかに達した後のモニター停止:
  - 収集制限(5項目)、または
  - タイムアウト(120秒)

**実行方法:**
```bash
cargo run --example monitor_with_ttl
```

**期待される動作:** このサンプルは最大5つのJSONアップリンクメッセージを収集するか、120秒後に停止します(どちらか早い方)。

### 7. `monitor_with_handle.rs`

ハンドルによる外部制御を使用したリアルタイム監視で、pause/resume機能を示します。

**デモ内容:**
- 外部制御用のハンドル付きでモニターを開始
- モニターの一時停止(コールバック処理が停止、データはルーター側でバッファリング)
- モニターの再開(バッファリングされたデータを受信)
- モニターの正常停止
- ハンドルベースの制御により柔軟な監視ワークフローが可能

**実行方法:**
```bash
cargo run --example monitor_with_handle
```

**期待される動作:** モニターは5秒間実行され、3秒間一時停止し、5秒間再開してから停止します。一時停止中は出力が表示されませんが、ルーターはデータをバッファリングし続けます。

### 8. `monitor_with_callback_and_handle.rs`

コールバック処理とハンドルによるpause/resume制御を組み合わせます。

**デモ内容:**
- コールバックとハンドルの両方でモニターを開始
- コールバックで各JSON行を処理
- 受信メッセージ数のカウント
- データがバッファリングされている間、コールバック処理を一時停止
- コールバック処理の再開
- コールバックベース監視の外部制御

**実行方法:**
```bash
cargo run --example monitor_with_callback_and_handle
```

**期待される動作:** 3メッセージを受信し、3秒間一時停止(コールバックは呼び出されない)、再開して2メッセージを追加受信してから停止します。合計5メッセージが処理されます。

## よくある問題

### "No such file or directory" または "bjig not found"

`bjig` CLIがインストールされており、PATHに含まれているか、場所を指定していることを確認してください:

```bash
export BJIG_CLI_BIN_PATH=/path/to/bjig
```

### シリアルポートで "Permission denied"

Linux/macOSでは、ユーザーをdialout/uucpグループに追加する必要がある場合があります:

```bash
# Linux
sudo usermod -a -G dialout $USER

# macOS
sudo dseditgroup -o edit -a $USER -t user uucp
```

変更を有効にするには、ログアウトして再度ログインしてください。

### "No such device" エラー

BraveJIGルーターが接続されていることを確認し、正しいポートをチェックしてください:

```bash
# Linux/macOS - 利用可能なポートを一覧表示
ls /dev/tty*

# macOS固有
ls /dev/cu.*
```

## 学習パス

ライブラリを理解するための推奨順序:

1. `env_config.rs` から始めて設定を理解する
2. `simple_router.rs` で基本操作を試す
3. センサーが接続されている場合は `module_control.rs` を試す
4. `restart_router.rs` で再起動シーケンスを学ぶ
5. `monitor.rs` でリアルタイムデータ収集を探索する
6. `monitor_with_ttl.rs` でタイムアウトベースの監視を学ぶ
7. 上級: `monitor_with_handle.rs` でpause/resume機能を使った外部制御を学ぶ
8. 上級: `monitor_with_callback_and_handle.rs` でコールバックベース監視と制御を学ぶ

## 追加リソース

- [メインREADME](../README.md) - ライブラリドキュメント
- [APIドキュメント](https://docs.rs/bjig_controller) - 完全なAPIリファレンス
- [bjig CLIドキュメント](https://github.com/MONO-ON/bravejig) - BraveJIG CLIツール
