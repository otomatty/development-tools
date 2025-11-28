# TODO Collector GUI 実装作業ログ

**日付**: 2025-11-28
**関連 Issue**: GitHub Issue #1 "feat: TODO Collector ツールの実装"
**ステータス**: 完了

---

## 実施した作業

### 1. 現状調査

- GitHub Issue #1 の要件を確認
- todo-collector CLI ツールが既に `/tools/todo-collector/` に実装済みであることを確認
- CLI ツールの動作確認（正常に JSON 出力）

### 2. GUI 対応の問題特定

- `ResultView` コンポーネントが `shai-hulud-scanner` 専用にハードコードされていることを発見
- `tool.json` の `resultParser.schema` を活用した汎用的な表示が必要

### 3. ResultView コンポーネントの改修

**ファイル**: `/src/components/result_view.rs`

- `get_value_by_path()` 関数を追加（JSONPath ライクなパス解析）
- `SummaryViewInner` を schema.summary に基づいて動的レンダリングするよう修正
- `DetailsViewInner` を schema.details.columns に基づいて動的レンダリングするよう修正
- タイプ別バッジカラーリング（FIXME: red, TODO: blue, HACK: yellow, NOTE: green）

### 4. アイコン追加

**ファイル**: `/src/components/icons.rs`

追加したアイコン：

- `check-square` - TODO 用
- `tool` - HACK 用
- `file-text` - NOTE 用

### 5. CSS スタイル追加

**ファイル**: `/input.css`

追加したクラス：

- `.badge-info` - 青色バッジ（`bg-blue-500/20 text-blue-400`）

---

## 変更ファイル一覧

| ファイル                        | 変更内容                       |
| ------------------------------- | ------------------------------ |
| `src/components/result_view.rs` | スキーマベースの汎用表示に改修 |
| `src/components/icons.rs`       | 3 つのアイコン追加             |
| `input.css`                     | badge-info クラス追加          |

---

## 動作確認

- [x] `cargo check` - コンパイル成功（警告のみ）
- [x] `trunk build` - WASM ビルド成功
- [x] `cargo tauri dev` - アプリ起動成功

---

## 技術的なポイント

### JSONPath ライクなパス解析

`tool.json` の schema 定義例：

```json
{
  "path": "$.summary.by_type.TODO",
  "countType": "value"
}
```

これを解析する `get_value_by_path()` 関数を実装：

```rust
fn get_value_by_path(json: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    // ルート要素自体を返すケース
    if path == "$" {
        return Some(json.clone());
    }
    
    // $. で始まるパスを正規化
    let path = path.strip_prefix("$.").unwrap_or(path);
    
    // 空パスの場合はルート要素を返す
    if path.is_empty() {
        return Some(json.clone());
    }
    
    let mut current = json;
    // 空のパス部分をフィルタリング（連続ドット対策）
    for part in path.split('.').filter(|p| !p.is_empty()) {
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(part)?;
            }
            serde_json::Value::Array(arr) => {
                let idx: usize = part.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current.clone())
}
```

### Leptos の型統一

if/else で異なるビュー型を返す場合は `.into_any()` で統一：

```rust
if schema.is_some() {
    // 動的レンダリング
    view! { ... }.into_any()
} else {
    // フォールバック
    view! { ... }.into_any()
}
```

---

## 次のステップ

- [ ] GUI での実際の動作確認（TODO Collector 選択 → 実行 → 結果表示）
- [ ] Issue #1 のクローズ
- [ ] 追加のツールでも schema ベース表示が機能することを確認

---

**作業者**: AI (Claude)
**最終更新**: 2025-11-28
