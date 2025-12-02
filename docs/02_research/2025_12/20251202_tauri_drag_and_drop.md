# Tauri WebView におけるドラッグ＆ドロップ実装の知見

## 概要

本ドキュメントは、Tauri（WebKit WebView）環境で Leptos を使用した Kanban ボードのドラッグ＆ドロップ機能を実装する際に得られた知見をまとめたものです。

## 問題の背景

### 初期実装（失敗）

当初、HTML5 の標準ドラッグ＆ドロップ API を使用して実装を試みました：

```rust
// ❌ HTML5 Drag API - Tauri では動作しない
<div
    draggable="true"
    on:dragstart=move |e| { /* ... */ }
    on:dragover=move |e| { e.prevent_default(); }
    on:drop=move |e| { /* ... */ }
>
```

### 発生した問題

- `dragstart` イベントは発火するが、その後の `dragenter`、`dragover`、`drop` イベントが**一切発火しない**
- コンソールログで確認すると、ドラッグ開始のログのみ出力され、以降のイベントログが表示されない

### 根本原因

**Tauri の WebKit WebView は HTML5 ドラッグ＆ドロップイベントをブロックする**

これは Tauri の既知の制限であり、WebView のセキュリティ/サンドボックス機能に起因します。ブラウザ環境では正常に動作するコードが、Tauri 環境では動作しません。

## 解決策：マウスイベントベースの実装

HTML5 Drag API の代わりに、基本的なマウスイベント（`mousedown`, `mouseup`, `mousemove`, `mouseenter`, `mouseleave`）を使用して独自のドラッグ＆ドロップを実装しました。

### アーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                     KanbanBoard (Parent)                     │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Signals:                                                │ │
│  │  - dragging: Option<MouseDragState>                     │ │
│  │  - hover_column: Option<String>                         │ │
│  │  - mouse_pos: (i32, i32)                                │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Global Event Handlers (document level):                 │ │
│  │  - mousemove → update mouse_pos for ghost card          │ │
│  │  - mouseup → handle drop or cancel                      │ │
│  │  - selectstart → prevent text selection during drag     │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │ Column 1 │  │ Column 2 │  │ Column 3 │  │ Column N │     │
│  │mouseenter│  │mouseenter│  │mouseenter│  │mouseenter│     │
│  │mouseleave│  │mouseleave│  │mouseleave│  │mouseleave│     │
│  │          │  │          │  │          │  │          │     │
│  │ ┌──────┐ │  │ ┌──────┐ │  │ ┌──────┐ │  │ ┌──────┐ │     │
│  │ │Card  │ │  │ │Card  │ │  │ │Card  │ │  │ │Card  │ │     │
│  │ │mouse │ │  │ │mouse │ │  │ │mouse │ │  │ │mouse │ │     │
│  │ │down  │ │  │ │down  │ │  │ │down  │ │  │ │down  │ │     │
│  │ └──────┘ │  │ └──────┘ │  │ └──────┘ │  │ └──────┘ │     │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘     │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Ghost Card (fixed position, follows mouse_pos)          │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 状態管理

```rust
/// ドラッグ状態を保持する構造体
#[derive(Clone, Debug, PartialEq)]
pub struct MouseDragState {
    pub issue_number: i32,      // ドラッグ中のIssue番号
    pub from_status: String,    // 元のステータス（カラム）
    pub issue_title: String,    // ゴーストカード表示用のタイトル
}

// メインコンポーネントで3つのシグナルを管理
let (dragging, set_dragging) = signal(Option::<MouseDragState>::None);
let (hover_column, set_hover_column) = signal(Option::<String>::None);
let (mouse_pos, set_mouse_pos) = signal((0i32, 0i32));
```

### イベントフロー

```
1. mousedown on Card
   └─→ set_dragging(Some(MouseDragState))
   └─→ ドラッグ開始

2. mousemove (global)
   └─→ set_mouse_pos((x, y))
   └─→ ゴーストカードが追従

3. mouseenter on Column
   └─→ set_hover_column(Some(column_id))
   └─→ ドロップ先ハイライト表示

4. mouseleave on Column
   └─→ set_hover_column(None)
   └─→ ハイライト解除

5. mouseup (global)
   └─→ if dragging && hover_column
   │   └─→ ステータス変更を実行
   └─→ set_dragging(None)
   └─→ ドラッグ終了
```

## 実装の詳細

### 1. グローバルイベントハンドラの設定

`Effect` を使用して、document レベルでイベントリスナーを登録します：

```rust
Effect::new(move |_| {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    let document = web_sys::window().unwrap().document().unwrap();

    // マウス位置の追跡（ゴーストカード用）
    let mousemove_handler = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
        set_mouse_pos.set((e.client_x(), e.client_y()));
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("mousemove", mousemove_handler.as_ref().unchecked_ref())
        .unwrap();

    // ドロップ処理（グローバル mouseup）
    let mouseup_handler = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
        if let Some(drag_state) = dragging.get_untracked() {
            if let Some(target_status) = hover_column.get_untracked() {
                if drag_state.from_status != target_status {
                    // ステータス変更を実行
                    status_change_signal.set(Some(StatusChangeEvent {
                        issue_number: drag_state.issue_number,
                        new_status: target_status,
                    }));
                }
            }
            set_dragging.set(None);
        }
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("mouseup", mouseup_handler.as_ref().unchecked_ref())
        .unwrap();

    // テキスト選択の防止
    let selectstart_handler = Closure::wrap(Box::new(move |e: web_sys::Event| {
        if dragging.get_untracked().is_some() {
            e.prevent_default();
        }
    }) as Box<dyn FnMut(_)>);

    document
        .add_event_listener_with_callback("selectstart", selectstart_handler.as_ref().unchecked_ref())
        .unwrap();

    // Closure をリークさせて生存させる
    // TODO: [DEBT] on_cleanup で適切にクリーンアップする
    mousemove_handler.forget();
    mouseup_handler.forget();
    selectstart_handler.forget();
});
```

### 2. ドラッグ開始（カードの mousedown）

```rust
<div
    draggable="false"  // ネイティブドラッグを無効化
    on:selectstart=move |e| { e.prevent_default(); }  // テキスト選択防止
    on:mousedown=move |e: web_sys::MouseEvent| {
        e.prevent_default();
        if e.button() == 0 {  // 左クリックのみ
            set_dragging.set(Some(MouseDragState {
                issue_number,
                from_status: current_status.clone(),
                issue_title: title.clone(),
            }));
        }
    }
>
```

### 3. ホバーカラムの追跡

```rust
<div
    on:mouseenter=move |_| {
        if is_dragging() {
            set_hover_column.set(Some(status_value.to_string()));
        }
    }
    on:mouseleave=move |_| {
        if is_dragging() {
            set_hover_column.set(None);
        }
    }
>
```

### 4. ゴーストカード（視覚的フィードバック）

マウスに追従する半透明のカードを表示：

```rust
<Show when=move || dragging.get().is_some()>
    {move || {
        let drag_state = dragging.get();
        let (x, y) = mouse_pos.get();
        view! {
            <div
                class="fixed pointer-events-none z-50 bg-gray-800 rounded-lg p-3
                       shadow-2xl border-2 border-gm-accent-cyan w-64 opacity-90"
                style=move || format!(
                    "left: {}px; top: {}px; transform: translate(-50%, -50%);",
                    x, y
                )
            >
                <span class="text-xs text-gray-400">
                    {"#"}{drag_state.as_ref().map(|d| d.issue_number).unwrap_or(0)}
                </span>
                <p class="text-sm text-white line-clamp-2">
                    {drag_state.as_ref().map(|d| d.issue_title.clone()).unwrap_or_default()}
                </p>
            </div>
        }
    }}
</Show>
```

### 5. ドロップ先ハイライト

4 つの視覚的状態を実装：

```rust
class=move || {
    let dragging_active = is_dragging();
    let valid_target = is_valid_drop_target();  // ドラッグ元以外
    let hovered = is_hovered();  // 現在ホバー中

    format!(
        "flex flex-col w-72 rounded-lg border-2 transition-all duration-200 {}",
        if dragging_active && valid_target && hovered {
            // ホバー中の有効なドロップ先 - 内側グロー効果
            "bg-gm-accent-cyan/20 border-gm-accent-cyan
             shadow-[inset_0_0_20px_rgba(0,255,255,0.3)]
             ring-2 ring-gm-accent-cyan/50 ring-inset"
        } else if dragging_active && valid_target {
            // 有効なドロップ先（ホバーなし）- 破線ボーダー
            "bg-slate-900/50 border-gm-accent-cyan/50 border-dashed"
        } else if dragging_active && !valid_target {
            // ドラッグ元カラム - 半透明
            "bg-slate-900/30 border-slate-700/30 opacity-60"
        } else {
            // 通常状態
            "bg-slate-900/50 border-slate-700/50"
        },
    )
}
```

## 実装時の注意点

### 1. Closure のライフタイム管理

```rust
// ❌ 悪い例：Closure がスコープ外で破棄される
let handler = Closure::wrap(Box::new(move |e| { ... }) as Box<dyn FnMut(_)>);
document.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref()).unwrap();
// handler がここで破棄され、イベントが動作しなくなる

// ✅ 良い例：forget() でリークさせる
handler.forget();
// または on_cleanup で適切にクリーンアップ
```

### 2. テキスト選択の防止

ドラッグ中にテキストが選択されてしまう問題の対策：

```rust
// 方法1: selectstart イベントを防止（グローバル）
document.add_event_listener_with_callback("selectstart", ...);

// 方法2: 要素に preventDefault を設定
on:selectstart=move |e| { e.prevent_default(); }

// 方法3: CSS で選択不可に
class="select-none"

// 方法4: draggable 属性を明示的に無効化
draggable="false"
```

### 3. ReadSignal vs WriteSignal

```rust
// ❌ エラー：WriteSignal には .get() メソッドがない
let value = write_signal.get();

// ✅ 正しい：ReadSignal を使用
let (read_signal, write_signal) = signal(initial_value);
let value = read_signal.get();
```

### 4. イベント伝播の制御

カード内のクリック可能な要素（リンク、ボタン）がドラッグを開始しないようにする：

```rust
// リンクやボタンでは mousedown の伝播を停止
<a
    href=issue_url
    on:mousedown=move |e| e.stop_propagation()
    on:click=move |e| e.stop_propagation()
>
```

## パフォーマンスに関する考慮事項

### mousemove イベントの最適化

`mousemove` は非常に頻繁に発火するため、必要な場合のみ処理を行う：

```rust
// 現在の実装：常にマウス位置を更新
// ゴーストカードの滑らかな追従のために必要

// 最適化案（必要に応じて）：
// - requestAnimationFrame でスロットリング
// - ドラッグ中のみリスナーを登録/解除
```

## 今後の改善点

1. **イベントリスナーのクリーンアップ**

   - 現在は `forget()` でリークさせている
   - `on_cleanup` で適切に削除すべき

2. **タッチイベント対応**

   - `touchstart`, `touchmove`, `touchend` を追加
   - モバイル/タブレット対応

3. **アクセシビリティ**

   - キーボードでのドラッグ操作
   - ARIA 属性の追加

4. **アニメーション改善**
   - ドロップ時のアニメーション
   - カードの並び替えアニメーション

## 参考リンク

- [Tauri WebView の制限](https://github.com/nicholaslee119/tauri-dnd-example)
- [Leptos ドキュメント](https://leptos.dev/)
- [wasm-bindgen ドキュメント](https://rustwasm.github.io/wasm-bindgen/)

## 結論

Tauri 環境では HTML5 Drag API が動作しないため、マウスイベントを使用した独自実装が必要です。この実装は以下の利点があります：

- **完全な制御**: ドラッグ動作を細かくカスタマイズ可能
- **クロスプラットフォーム**: Tauri WebView で確実に動作
- **リッチな UI**: ゴーストカード、ハイライトなどの視覚的フィードバック

一方で、イベントリスナーの管理やテキスト選択の防止など、追加の考慮が必要になります。
