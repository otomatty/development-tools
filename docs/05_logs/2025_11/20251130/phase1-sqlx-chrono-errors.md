# Phase 1 実装時のSQLx chronoエラー解決記録

**日付**: 2025-11-30  
**Issue**: #74 (コントリビューショングラフ強化 & コード行数視覚化機能)  
**フェーズ**: Phase 1 - バックエンド基盤実装

---

## 発生したエラー

### 概要

Phase 1の実装中、`chrono::DateTime<Utc>`および`chrono::NaiveDate`型をSQLxのクエリで使用しようとした際に、約30件のコンパイルエラーが発生しました。

### エラーメッセージ

```
error[E0277]: the trait bound `chrono::DateTime<Utc>: Type<Sqlite>` is not satisfied
error[E0277]: the trait bound `chrono::NaiveDate: Type<Sqlite>` is not satisfied
error[E0277]: the trait bound `chrono::DateTime<Utc>: Decode<'_, Sqlite>` is not satisfied
```

### 問題のコード（修正前）

```rust
// models/code_stats.rs
pub struct DailyCodeStats {
    pub date: NaiveDate,  // ❌ SQLiteでデコードできない
    pub last_sync_at: DateTime<Utc>,  // ❌ SQLiteでデコードできない
    // ...
}

// repository/code_stats.rs
pub async fn get_daily_code_stats_range(
    &self,
    user_id: i64,
    start_date: NaiveDate,  // ❌ SQLiteにバインドできない
    end_date: NaiveDate,    // ❌ SQLiteにバインドできない
) -> DbResult<Vec<DailyCodeStats>> {
    sqlx::query(...)
        .bind(start_date)  // エラー
        .bind(end_date)    // エラー
        // ...
}
```

---

## 原因

SQLxの`chrono`サポートはオプション機能として提供されており、`Cargo.toml`で明示的に有効にする必要があります。

```toml
# この設定がない場合、DateTime/NaiveDate型は使用できない
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "chrono"] }
```

このプロジェクトでは`chrono` featureが有効になっておらず、既存のコードベースではDateTime/NaiveDateをString型として保存し、必要に応じてパースするパターンが採用されていました。

---

## 解決策

### 1. モデル層での変更

DateTime/NaiveDate型の代わりにString型を使用し、ヘルパーメソッドで変換を行う：

```rust
// models/code_stats.rs
pub struct DailyCodeStats {
    pub date: String,  // "YYYY-MM-DD"形式で保存
    pub last_sync_at: String,  // RFC3339形式で保存
    // ...
}

impl DailyCodeStats {
    /// 日付をNaiveDateとしてパース
    pub fn date_as_naive(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d").ok()
    }
}

impl SyncMetadata {
    /// last_sync_atをDateTimeとしてパース
    pub fn last_sync_at_parsed(&self) -> Option<DateTime<Utc>> {
        self.last_sync_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }
}
```

### 2. Repository層での変更

日付をバインドする前にString形式に変換：

```rust
// repository/code_stats.rs
pub async fn get_daily_code_stats_range(
    &self,
    user_id: i64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> DbResult<Vec<DailyCodeStats>> {
    // NaiveDateをStringに変換してからバインド
    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();
    
    sqlx::query(...)
        .bind(&start_str)  // ✅ String型なのでバインド可能
        .bind(&end_str)    // ✅ String型なのでバインド可能
        // ...
}
```

### 3. Commands層での変更

DateTimeをRFC3339文字列に変換して保存：

```rust
// commands/github.rs
pub async fn sync_code_stats(...) -> Result<CodeStatsResponse, String> {
    // ...
    let now = Utc::now().to_rfc3339();  // DateTimeをStringに変換
    
    db.update_sync_metadata(
        user_id,
        "code_stats",
        &now,  // String型で渡す
        None,
    ).await?;
    // ...
}
```

---

## 既存コードベースのパターン

このプロジェクトでは、以下の箇所で同様のパターンが使用されています：

1. **`UserRow`** (`repository/user.rs`)
   - `token_expires_at: Option<String>`
   - `created_at: String`
   - `updated_at: String`

2. **`UserStatsRow`** (`repository/user_stats.rs`)
   - `last_activity_date: Option<String>`
   - `updated_at: String`

3. **`UserSettingsRow`** (`repository/settings.rs`)
   - `created_at: String`
   - `updated_at: String`

---

## 代替案（採用しなかった理由）

### chronoフィーチャーを有効にする

```toml
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "chrono"] }
```

**採用しなかった理由**:
- 既存コードベースとの整合性を保つため
- 既存のテストやロジックへの影響を最小化するため
- プロジェクト全体のアーキテクチャ方針に従うため

---

## 教訓

1. **既存コードのパターンを最初に確認する**: 新機能実装前に、同様の処理が既存コードでどのように実装されているか確認することで、多くのエラーを事前に回避できる

2. **SQLxのフィーチャーフラグを確認する**: SQLxは多くのオプション機能を持っており、特定の型サポートにはフィーチャーフラグの有効化が必要

3. **String型での日付保存は有効なパターン**: SQLiteはネイティブな日付型を持たないため、String形式での保存は合理的な選択

---

## 関連ファイル

- `src-tauri/src/database/models/code_stats.rs` - データモデル
- `src-tauri/src/database/repository/code_stats.rs` - リポジトリ操作
- `src-tauri/src/commands/github.rs` - Tauriコマンド
- `src-tauri/Cargo.toml` - SQLx依存関係設定
