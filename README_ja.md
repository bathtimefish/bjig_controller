# bjig_controller

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.0-orange)](https://crates.io/crates/bjig_controller)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

[English](README.md) | [日本語](README_ja.md)

`bjig` CLIコマンドを介してBraveJIG USBルーターを制御するRustライブラリ

## 概要

`bjig_controller`は、BraveJIG USBルーターとセンサーモジュールを制御するための高レベルで型安全なRust APIを提供します。`bjig` CLIバイナリをラップすることで、RustアプリケーションにBraveJIG機能を簡単に統合できます。

### BraveJIGとは？

BraveJIGは、USBルーターと無線センサーモジュールで構成されるIoTゲートウェイシステムです。このライブラリは、すべてのルーターおよびモジュール操作にプログラムからアクセスできるようにします。

## 特徴

- **ルーター制御**: ルーターの起動/停止、ファームウェア管理、設定変更
- **モジュール管理**: センサーモジュールの制御、データ取得、ファームウェア更新
- **リアルタイムモニタリング**: ルーターとモジュールのイベント監視
- **環境変数サポート**: 環境変数からの自動設定
- **型安全なAPI**: 強く型付けされたレスポンスと包括的なエラーハンドリング
- **Async/Await**: 効率的な非同期操作のため`tokio`上に構築

## インストール

`Cargo.toml`に以下を追加してください：

```toml
[dependencies]
bjig_controller = "0.1"
tokio = { version = "1", features = ["full"] }
```

## クイックスタート

```rust
use bjig_controller::BjigController;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // コントローラーを作成（環境変数から読み込み）
    let bjig = BjigController::from_env()?;

    // ルーターのバージョンを取得
    let version = bjig.router().get_version().await?;
    println!("ルーターバージョン: {}", version.version);

    // モジュールからセンサーデータを取得
    let data = bjig.module("0121", "2468800203400004")
        .instant_uplink()
        .await?;
    println!("センサーデータ: {:?}", data);

    Ok(())
}
```

## 環境変数

ライブラリは以下の環境変数をサポートしています：

| 環境変数 | 説明 | デフォルト値 |
|---------|------|-------------|
| `BJIG_CLI_BIN_PATH` | bjigバイナリのパス | `./bin/bjig` |
| `BJIG_CLI_PORT` | シリアルポート（例：`/dev/ttyACM0`, `COM3`） | *(必須)* |
| `BJIG_CLI_BAUD` | ボーレート | `38400` |
| `BJIG_CLI_MODULE_CONFIG` | モジュール設定ファイルのパス | `module-config.yml` |

### 使用例

```bash
# 環境変数を設定
export BJIG_CLI_BIN_PATH=./bin/bjig
export BJIG_CLI_PORT=/dev/ttyACM0
export BJIG_CLI_BAUD=115200

# アプリケーションを実行
cargo run
```

## 使い方

### コントローラーの作成

ニーズに応じて、`BjigController`を初期化する方法がいくつかあります：

#### 1. 環境変数から（推奨）

環境変数から自動的に設定を読み込みます：

```rust
let bjig = BjigController::from_env()?;
```

このメソッドは以下を読み込みます：
- `BJIG_CLI_BIN_PATH` → bjigバイナリパス（デフォルト: `./bin/bjig`）
- `BJIG_CLI_PORT` → シリアルポート（必須）
- `BJIG_CLI_BAUD` → ボーレート（デフォルト: `38400`）

#### 2. 完全な明示的設定

すべての設定をコードで直接指定します：

```rust
let bjig = BjigController::new("./bin/bjig")?
    .with_port("/dev/ttyACM0")
    .with_baud(115200);
```

#### 3. 最小限の設定（ポートのみ）

ポートのみを指定し、他はすべてデフォルトを使用します：

```rust
let bjig = BjigController::new("./bin/bjig")?
    .with_port("/dev/ttyACM0");
// ボーレートはデフォルトの38400
```

#### 4. ハイブリッドアプローチ

環境変数と明示的な上書きを組み合わせます：

```rust
// 環境変数を使用しつつ特定の設定を上書き
let bjig = BjigController::from_env()?
    .with_baud(115200)
    .with_port("/dev/ttyACM1");
```

#### 5. カスタムバイナリパス

カスタムbjigバイナリの場所を使用します：

```rust
let bjig = BjigController::new("/usr/local/bin/bjig")?
    .with_port("/dev/ttyACM0")
    .with_baud(115200);
```

#### 6. モジュール設定ファイル付き

カスタムモジュール設定ファイルを指定します：

```rust
let bjig = BjigController::from_env()?
    .with_module_config_path("/etc/bjig/custom-modules.yml");
```

### 設定の優先順位

同じ設定が複数の場所で指定されている場合、優先順位は以下の通りです：

1. **明示的なメソッド呼び出し**（`.with_port()`, `.with_baud()`） - 最高優先度
2. **コントローラーのデフォルト**（ビルダーメソッドで設定）
3. **環境変数**（`from_env()`使用時）
4. **組み込みデフォルト**（ボーレート38400、バイナリパス`./bin/bjig`）

例：

```rust
// BJIG_CLI_PORT=/dev/ttyACM0 (環境変数)
// BJIG_CLI_BAUD=9600 (環境変数)

let bjig = BjigController::from_env()?
    .with_baud(115200);  // ボーレートを115200に上書き

// 結果: port=/dev/ttyACM0 (環境変数から), baud=115200 (明示的)

### ルーターコマンド

```rust
// ファームウェアバージョンの取得
let version = bjig.router().get_version().await?;

// ルーターの起動/停止
bjig.router().start().await?;
bjig.router().stop().await?;

// モジュールIDの取得
let modules = bjig.router().get_module_id(None).await?; // 全モジュール
let module = bjig.router().get_module_id(Some(0)).await?; // 特定のインデックス

// スキャンモードの設定
use bjig_controller::ScanModeType;
bjig.router().set_scan_mode(ScanModeType::LongRange).await?;

// モジュールIDの削除
bjig.router().remove_module_id(Some(0)).await?; // インデックス0を削除
bjig.router().remove_module_id(None).await?; // すべて削除

// キープアライブ信号
bjig.router().keep_alive().await?;

// サポートされているセンサーの取得（シリアル接続不要）
let sensors = bjig.router().get_supported_sensor_id()?;

// ルーターファームウェアの更新
bjig.router().dfu("router_firmware.bin").await?;
```

### モジュールコマンド

```rust
let module = bjig.module("0121", "2468800203400004");

// 即時センサーデータの取得
let data = module.instant_uplink().await?;

// パラメータの取得/設定
let params = module.get_parameter().await?;
module.set_parameter(&json!({"interval": 60})).await?;

// モジュールの再起動
module.restart().await?;

// モジュールファームウェアの更新
module.dfu("module_firmware.bin").await?;

// モジュール固有の制御コマンド
module.control(&json!({"clear_counts": "all"})).await?;
```

### モニターコマンド

モニターコマンドは、ルーターからのJSON形式のアップリンクデータをリアルタイムでストリーミングします。

#### 基本的な使い方

```rust
// 無期限で監視（Ctrl+Cまで）
bjig.monitor().start().await?;

// タイムアウト付きで監視（60秒間）
bjig.monitor().start_with_ttl(60).await?;
```

#### コールバックを使った高度な使い方

監視プロセスをより細かく制御するために、到着した各JSON行を処理できるコールバックベースのメソッドを使用できます：

```rust
// 各JSON行を収集して処理
let mut json_list: Vec<String> = Vec::new();

bjig.monitor().start_with_callback(|line| {
    // 各行を即座に表示
    println!("受信: {}", line);

    // コレクションに追加
    json_list.push(line.to_string());

    // Ok(true)で継続、Ok(false)で停止
    Ok(json_list.len() < 5)  // 5件で停止
}).await?;

println!("{}件を収集しました", json_list.len());
```

```rust
// タイムアウトとコールバック併用
let mut json_list: Vec<String> = Vec::new();

bjig.monitor().start_with_ttl_and_callback(120, |line| {
    println!("受信: {}", line);
    json_list.push(line.to_string());
    Ok(json_list.len() < 5)  // 5件または120秒で停止
}).await?;
```

**コールバックメソッド:**
- `start_with_callback(callback)` - コールバック付き監視、タイムアウトなし
- `start_with_callback_on(port, baud, callback)` - 特定のポートでコールバック付き監視
- `start_with_ttl_and_callback(ttl_secs, callback)` - タイムアウトとコールバック併用

コールバック関数は各JSON行を`&str`として受け取り、`Result<bool>`を返す必要があります：
- `Ok(true)`を返すと監視を継続
- `Ok(false)`を返すと監視を停止してプロセスを終了
- `Err(e)`を返すとエラーで停止

### ポートのオーバーライド

すべてのコマンドは、オプションでポート/ボーレートのオーバーライドをサポートしています：

```rust
// デフォルトのポート/ボーレートを使用
let version = bjig.router().get_version().await?;

// 特定のコマンドでのみオーバーライド
let version = bjig.router()
    .get_version_on(Some("/dev/ttyACM1"), Some(9600))
    .await?;
```

## サンプルコード

`examples/` ディレクトリに完全なサンプルコードがあります：

- **`simple_router.rs`** - ルーターの基本操作（バージョン、スキャンモード、モジュール一覧）
- **`module_control.rs`** - モジュール管理（インスタントアップリンク、パラメータ）
- **`env_config.rs`** - 環境変数の設定例
- **`restart_router.rs`** - タイミング制御付きのルーター再起動シーケンス
- **`monitor.rs`** - アップリンクデータの連続監視（Ctrl+Cまで）
- **`monitor_with_ttl.rs`** - タイムアウト付きアップリンクデータ監視

サンプルの実行方法：

```bash
# 最初に環境変数を設定
export BJIG_CLI_PORT=/dev/ttyACM0
export BJIG_CLI_BAUD=115200

# サンプルを実行
cargo run --example simple_router
cargo run --example restart_router
cargo run --example monitor              # Ctrl+Cで停止
cargo run --example monitor_with_ttl     # 30秒後に自動停止
```

## エラーハンドリング

ライブラリはすべての操作で`Result<T, BjigError>`を使用します：

```rust
match bjig.router().get_version().await {
    Ok(version) => println!("バージョン: {}", version.version),
    Err(e) => eprintln!("エラー: {}", e),
}
```

主なエラータイプ：

- `BinaryNotFound` - 指定されたパスにbjigバイナリが見つからない
- `CommandFailed` - コマンド実行が失敗
- `PortNotConfigured` - シリアルポートが設定されていない
- `JsonParseError` - コマンド出力のパースに失敗
- `FileNotFound` - ファームウェアファイルが見つからない

## シリアルポートの排他性

bjigコマンドはシリアルポート通信を使用するため、本質的に排他的です。一度に1つのプロセスのみがシリアルポートに接続できます。同じポートで複数のbjig_controllerインスタンスを同時に実行しようとすると、2番目のインスタンスは「connection busy」エラーで失敗します。

これはハードウェアの制限であり、ライブラリの制限ではありません。

## 開発

### ビルド

```bash
cargo build
```

### テスト

```bash
cargo test
```

### ドキュメント

```bash
cargo doc --open
```

## 必要要件

- **Rust**: 1.70以降
- **bjig CLI**: バイナリは`./bin/bjig`に含まれています
- **ハードウェア**: BraveJIG USBルーターとセンサーモジュール
- **OS**: Linux、macOS、Windows（tokioがサポートするプラットフォーム）

## アーキテクチャ

```
┌─────────────────────────┐
│   あなたのRustアプリ    │
│                         │
├─────────────────────────┤
│   bjig_controller       │  ← このライブラリ（型安全なAPI）
│   - ルーターコマンド    │
│   - モジュールコマンド  │
│   - モニターコマンド    │
├─────────────────────────┤
│   bjig CLIバイナリ      │  ← コマンドラインツール
│   (プロセス実行)        │
├─────────────────────────┤
│   BraveJIGルーター      │  ← ハードウェア（USB接続）
│   + センサーモジュール  │
└─────────────────────────┘
```

## コントリビューション

コントリビューションを歓迎します！Pull Requestをお気軽に送ってください。

## ライセンス

MITライセンス - 詳細はLICENSEファイルを参照してください

## 作者

**bathtimefish**

## リンク

- **リポジトリ**: https://github.com/bathtimefish/bjig_controller
- **ドキュメント**: https://docs.rs/bjig_controller
- **Crates.io**: https://crates.io/crates/bjig_controller
- **Issues**: https://github.com/bathtimefish/bjig_controller/issues

## 関連プロジェクト

- **bjig_cli_rust**: 公式BraveJIG CLIツール（Rust実装）
- **bjig_cli_python**: 公式BraveJIG CLIツール（Python実装）
