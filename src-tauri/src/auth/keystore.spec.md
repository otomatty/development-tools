# Token Encryption — Security Model

Issue #196 / 監査レポート §9.3 に対応する、アクセストークンの保管モデル仕様。

## 背景

GitHub Device Flow から取得した `access_token` は SQLite (`users.access_token_encrypted`) に
保存される。
監査前は AES-256-GCM の鍵をアプリ識別子 `com.sugaiakimasa.development-tools` から
ハッシュ派生 (`Crypto::from_app_key`) しており、

- 鍵がバイナリから再生成可能
- ローカル DB ファイルだけ流出すれば、誰でも復号できる

という状態だった。本仕様は鍵を **OS のキーストア (Keychain / DPAPI / Secret Service)** に
退避し、at-rest プロテクションを強化する。

## 関連ファイル

| 役割 | パス |
|------|------|
| キーストア抽象 | `src-tauri/src/auth/keystore.rs` |
| トークン暗号化 | `src-tauri/src/auth/crypto.rs` |
| トークン管理 + マイグレーション | `src-tauri/src/auth/token.rs` |
| 起動時マイグレーション呼び出し | `src-tauri/src/lib.rs` |
| スキーマ (encryption_version 列) | `src-tauri/src/database/migrations.rs` (v17) |

## アーキテクチャ

```
┌────────────────────────────────────────────────────────────────────┐
│ Process memory                                                     │
│                                                                    │
│  ┌──────────────┐   get_or_create_key   ┌──────────────────────┐   │
│  │ TokenManager │ ────────────────────▶ │  KeyStore (trait)    │   │
│  │              │                       │  ├ OsKeyStore         │   │
│  │ Crypto (AES) │  ◀── 32-byte key ──── │  │  ├ Keychain        │   │
│  │              │                       │  │  ├ DPAPI / WCM     │   │
│  └──────┬───────┘                       │  │  └ Secret Service  │   │
│         │                               │  └ MemoryKeyStore     │   │
│         │ encrypt / decrypt             │     (tests only)      │   │
│         ▼                               └──────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ SQLite: users.access_token_encrypted (base64 nonce+cipher)   │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

### 鍵の生成と保管

1. アプリ起動 → `TokenManager::new` → `Crypto::from_keystore(OsKeyStore)`
2. `OsKeyStore::get_key` が OS キーストアの `keyring` エントリを照会
   - `service = "com.sugaiakimasa.development-tools"`
   - `account = "token-encryption-master-key"`
3. 初回起動など未登録時は `rand::rng().fill(&mut [u8; 32])` で乱数 32 バイトを
   生成し、base64 エンコードして OS キーストアに保存
4. 以後はキーストアから読み出した鍵で AES-256-GCM を初期化

### プラットフォーム別バックエンド

| OS | キーストア | `keyring` クレートのバックエンド |
|----|------------|---------------------------------|
| macOS / iOS | Keychain (`SecKeychain`) | `apple-native` (`security-framework`) |
| Windows | Credential Manager (DPAPI 経由) | `windows-native` (`windows-sys`) |
| Linux / FreeBSD | Secret Service (GNOME Keyring / KWallet) | `sync-secret-service` + `crypto-rust` |

`vendored` 機能で DBus / OpenSSL を静的リンクし、エンドユーザー環境への依存を最小化。

### ヘッドレス Linux / CI

DBus + Secret Service が無い環境（一部 CI ランナーなど）では `OsKeyStore` の呼び出し
が失敗する。

- 単体テストは `MemoryKeyStore` を `TokenManager::with_keystore` に注入する形で
  キーストアを完全に避ける（`auth::token::tests` 参照）。
- 本番アプリは Secret Service が無いと **起動時に**エラーを返す方針。CI でアプリ全体を
  起動するテストを増やす場合は、`dbus-launch` を使うか、ヘッドレス用フォールバック
  (将来検討) を導入する。

## マイグレーション

### スキーマ変更

マイグレーション v17 (`add_encryption_version_to_users`) で `users` テーブルに
`encryption_version INTEGER NOT NULL DEFAULT 1` を追加。

| 値 | 意味 | 暗号鍵 |
|----|------|--------|
| `1` | 旧 (`Crypto::from_app_key`) | バイナリ由来の派生鍵 |
| `2` | OS キーストア管理鍵 | キーストアから取得した 32 バイト乱数 |

### 既存トークンの再暗号化

1. アプリ起動 → `lib.rs::setup` から `TokenManager::migrate_legacy_tokens_if_needed`
   を非同期タスクで実行
2. `encryption_version = 1` の行を全取得し、

   - 旧暗号で復号 → 新暗号で再暗号化 → `update_user_tokens` で UPDATE
   - 副作用として `encryption_version` を 2 に更新

3. 起動時スイープを取り逃した行（あるいはエラーで残った行）は、最初のトークン
   読み出し時に `TokenManager::decrypt_for_user` がオンデマンドで再暗号化する
4. 旧暗号で復号に失敗した行は **削除しない**。ログを残して放置し、将来の手当て
   に委ねる（強制ログアウトでユーザー体験を破壊しない）

### 冪等性

- すべての UPDATE は `encryption_version = 2` を無条件に書く
- 二重マイグレーション・並行実行が起きても、最終状態は (`v2`, 新ciphertext)
- 初回起動のスイープが空クエリならコストはほぼゼロ

### ログアウト済み行の扱い

ログアウトすると `access_token_encrypted = ''` になる。マイグレーションは復号
できないので、`encryption_version` のみを 2 に書き換えて legacy 分岐から外す
（`auth::token::tests::legacy_empty_row_is_tagged_without_failing` がガード）。

## セキュリティ境界

| 脅威 | 旧モデル | 新モデル |
|------|----------|----------|
| アプリ DB だけ流出 | **復号可能**（鍵はバイナリ由来） | 復号不可（鍵は OS キーストア） |
| バイナリだけ流出 | 影響なし | 影響なし |
| OS ログイン済みでローカル攻撃者がプロセスメモリ取得 | 復号可能 | 復号可能（変わらず） |
| OS キーストアへの正規アクセス権を持つ別アプリ | N/A | 復号可能（プラットフォームのポリシー次第） |
| マルチユーザー OS の他ユーザー | 復号可能（DB が読めれば） | OS キーストアの分離次第（macOS/Windows は通常分離、Linux は Secret Service の収集物次第） |

`Crypto::from_app_key` は `#[deprecated]` として残してあるが、**マイグレーション
専用**。新規の暗号化処理から呼ぶと clippy / 警告で気付ける。

## 鍵ローテーション

現状、明示的なローテーション API は提供しない。以下のシナリオで自動的に
ローテーションされる。

- **アンインストール → 再インストール**: OS キーストアの鍵は残るので、トークンも
  そのまま使える（再ログインは不要）。完全ワイプしたい場合はユーザーが OS の
  認証情報マネージャから削除する。
- **`reset_all_data` コマンド**: 既存ユーザーの DB を消すが、キーストアの鍵は
  そのまま。次回ログインで新規ユーザー行が同じ鍵で暗号化される。

将来的に鍵ローテーション機能を追加する場合は、`KeyStore::set_key` を使って
新鍵を保存し、全ユーザー行を旧鍵で復号 → 新鍵で再暗号化するパスを書く。

## マスター鍵喪失時のリカバリ

OS キーストアの鍵が消失した状態で SQLite DB だけ残ったケース（例: 認証情報
マネージャのクリーンアップツール、OS アカウントの移行、Linux Secret Service
のコレクション削除）への対応:

1. `TokenManager::with_keystore` 起動時に `KeyStore::get_key()` が `None` を
   返した場合、まず DB を `count_keystore_token_rows` でスキャン
2. `encryption_version = 2` かつ非空のトークン行が 1 件以上残っていれば、
   そのまま新鍵を生成すると **既存の暗号文すべてが永久に復号不能** になるため、
   `clear_keystore_orphan_tokens` でアクセストークンを空文字列に書き戻す
   （`logout()` と同じ semantics — XP / バッジ等の非機微データは保持）
3. その後に新しいランダム 32 バイト鍵を生成し OS キーストアへ保存
4. 次回起動時、ユーザーは Device Flow から再ログインを求められる

これにより「不可解な復号エラーがログに出続ける」状態を回避し、ユーザーに対して
クリーンな再認証フローを提供する。テストは
`auth::token::tests::lost_master_key_clears_orphan_token_rows_and_continues`
で挙動を担保している。

## テスト戦略

| テスト | 場所 | 目的 |
|--------|------|------|
| MemoryKeyStore round-trip | `auth::keystore::tests` | trait の最小契約を確認 |
| `Crypto::from_keystore` 永続性 | `auth::crypto::tests` | 2 個目の `Crypto` でも復号できる |
| 旧 `from_app_key` の互換性 | 同上 | マイグレーションが復号できることをガード |
| `legacy_row_is_decrypted_and_re_encrypted_in_place` | `auth::token::tests` | E2E: v1 行が v2 に書き換わり、旧鍵で復号できなくなる |
| `legacy_empty_row_is_tagged_without_failing` | 同上 | ログアウト済み行のバージョン昇格 |
| `save_tokens_writes_keystore_version` | 同上 | 新規ユーザーは初手から v2 |

### CI でのプラットフォームカバレッジ

- macOS / Windows: OS キーストアが標準で動くため `OsKeyStore` のスモークテスト
  を将来追加する余地あり（`keyring` 依存のためサンドボックス内で限定的）。
- Linux: Secret Service が無いと `OsKeyStore` は失敗する。ユニットテストは
  `MemoryKeyStore` 経由で実施しているため、CI 環境を問わずグリーンになる。
- GUI 全体の E2E（Tauri WebDriver など）が CI に乗ったら、`dbus-launch` で
  Secret Service を立ち上げる必要がある（未実装）。
