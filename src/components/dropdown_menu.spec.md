# DropdownMenu Component Specification

## Related Files

- Implementation: `src/components/dropdown_menu.rs`
- Tests: `src/components/dropdown_menu.rs` (inline tests)
- Parent Component: `src/components/home/profile_card.rs`

## Related Documentation

- Issue: `docs/01_issues/open/2025_11/20251129_06_dropdown-menu-for-actions.md`
- GitHub Issue: #39

## Requirements

### 責務

- 汎用的なドロップダウンメニュー UI を提供
- トリガー要素のクリックでメニューを開閉
- メニュー外クリックや ESC キーでメニューを閉じる
- アニメーション設定に応じたトランジション

### 状態構造

- `is_open: RwSignal<bool>` - メニューの開閉状態

### Props

#### DropdownMenu

| Prop       | Type                 | Default   | Description                      |
| ---------- | -------------------- | --------- | -------------------------------- |
| `trigger`  | `Fn() -> TriggerView`| required  | トリガーボタンの内容             |
| `children` | `Children`           | required  | メニューアイテム                 |
| `align`    | `&'static str`       | `"right"` | メニューの配置位置（right/left） |

#### DropdownMenuItem

| Prop       | Type                 | Default  | Description                  |
| ---------- | -------------------- | -------- | ---------------------------- |
| `on_click` | `Fn(ev::MouseEvent)` | required | クリックハンドラー           |
| `danger`   | `bool`               | `false`  | 危険なアクション（赤色表示） |
| `children` | `Children`           | required | アイテムの内容               |

### アクション

- `toggle_menu`: メニューの開閉をトグル
- `close_menu`: メニューを閉じる
- `handle_click_outside`: メニュー外クリック検出
- `handle_escape`: ESC キー検出

### スタイリング

#### トリガーボタン

```css
p-2 text-dt-text-sub hover:text-gm-accent-cyan transition-colors rounded-lg
```

#### メニューコンテナ

```css
absolute right-0 top-full mt-2 min-w-[160px]
bg-gm-bg-card/95 backdrop-blur-sm
border border-gm-accent-cyan/20 rounded-lg shadow-lg
z-50
```

#### メニューアイテム（通常）

```css
flex items-center gap-3 px-4 py-2
text-dt-text-main hover:bg-gm-accent-cyan/10
transition-colors cursor-pointer
```

#### メニューアイテム（danger）

```css
text-gm-error hover:bg-gm-error/10
```

#### アニメーション（有効時）

```css
opacity-0 -translate-y-2 → opacity-100 translate-y-0
transition-all duration-200
```

### アクセシビリティ

- トリガーボタンに `aria-expanded` 属性
- メニューに `role="menu"` 属性
- メニューアイテムに `role="menuitem"` 属性

## Test Cases

### TC-001: メニュー初期状態

- Given: DropdownMenu コンポーネントがマウントされた状態
- When: 初期表示時
- Then: メニューは閉じている（is_open = false）

### TC-002: トリガークリックでメニュー開く

- Given: メニューが閉じている状態
- When: トリガーボタンをクリック
- Then: メニューが開く（is_open = true）

### TC-003: トリガークリックでメニュー閉じる

- Given: メニューが開いている状態
- When: トリガーボタンをクリック
- Then: メニューが閉じる（is_open = false）

### TC-004: メニュー外クリックで閉じる

- Given: メニューが開いている状態
- When: メニュー外の領域をクリック
- Then: メニューが閉じる

### TC-005: ESC キーでメニュー閉じる

- Given: メニューが開いている状態
- When: ESC キーを押下
- Then: メニューが閉じる

### TC-006: メニューアイテムクリック

- Given: メニューが開いている状態
- When: メニューアイテムをクリック
- Then: on_click ハンドラーが呼ばれる、メニューが閉じる

### TC-007: アニメーション有効時のトランジション

- Given: AnimationContext でアニメーションが有効
- When: メニューを開く
- Then: フェードイン + スライドダウンアニメーションが適用される

### TC-008: アニメーション無効時の即時表示

- Given: AnimationContext でアニメーションが無効
- When: メニューを開く
- Then: アニメーションなしで即座に表示される

### TC-009: danger プロパティの適用

- Given: DropdownMenuItem に danger=true を設定
- When: メニューアイテムが表示される
- Then: 赤色のスタイル（text-gm-error）が適用される

### TC-010: aria-expanded 属性の更新

- Given: メニューの開閉状態が変化
- When: is_open が true/false に変更
- Then: トリガーボタンの aria-expanded 属性が対応する値に更新される
