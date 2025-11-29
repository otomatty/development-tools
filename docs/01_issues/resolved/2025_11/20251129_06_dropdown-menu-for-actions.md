# Issue: プロフィールカードのアクションを 3 ドットメニューに集約

## 概要

ダッシュボードのプロフィールカードにある「設定」「ログアウト」アイコンを、3 ドットメニュー（⋮）のドロップダウンに集約する。

## 現状

- `src/components/home/profile_card.rs` に設定ボタンとログアウトボタンが並列表示
- それぞれ独立したアイコンボタンとして配置
- UI 領域を占有している

## 要件

### 機能要件

1. **3 ドットメニューボタン**

   - 縦 3 ドット（⋮）アイコンのボタンを配置
   - クリックでドロップダウンメニューを表示

2. **ドロップダウンメニュー**

   - 設定（⚙️ Settings）
   - ログアウト（🚪 Logout）
   - メニュー外クリックで閉じる
   - ESC キーで閉じる

3. **アニメーション**
   - フェードイン/スライドダウンアニメーション
   - アニメーション無効設定時はアニメーションなし

### 技術要件

1. **DropdownMenu コンポーネント**

   - 再利用可能なドロップダウンメニューコンポーネント
   - トリガー要素とメニューアイテムを受け取る
   - 位置計算（トリガー要素の下に表示）

2. **状態管理**

   - メニューの開閉状態をローカル state で管理
   - クリックアウト検出

3. **アクセシビリティ**
   - キーボードナビゲーション対応
   - aria 属性の付与

## 影響範囲

### 新規ファイル

- `src/components/dropdown_menu.rs` - 汎用ドロップダウンメニューコンポーネント

### 修正ファイル

- `src/components/mod.rs` - dropdown_menu モジュールの公開
- `src/components/home/profile_card.rs` - ドロップダウンメニュー統合

## UI/UX デザイン

### Before

```
[Avatar] [Username]  [🔥 Streak] [⭐ Commits] [⚙️] [🚪]
```

### After

```
[Avatar] [Username]  [🔥 Streak] [⭐ Commits] [⋮]
                                              └─┬─┘
                                         ┌─────────────┐
                                         │ ⚙️ Settings │
                                         │ 🚪 Logout   │
                                         └─────────────┘
```

### ドロップダウンスタイル

```css
/* メニューコンテナ */
bg-gm-bg-card/95
backdrop-blur-sm
border border-gm-accent-cyan/20
rounded-lg
shadow-lg

/* メニューアイテム */
hover:bg-gm-accent-cyan/10
px-4 py-2
text-dt-text-main

/* Logoutアイテム */
text-gm-error hover:bg-gm-error/10
```

## テストケース

1. **TC-001**: 3 ドットボタンをクリックするとメニューが表示される
2. **TC-002**: メニュー外をクリックするとメニューが閉じる
3. **TC-003**: ESC キーでメニューが閉じる
4. **TC-004**: 「Settings」クリックで設定画面に遷移する
5. **TC-005**: 「Logout」クリックでログアウト処理が実行される
6. **TC-006**: アニメーション無効時に即座に表示/非表示される

## 優先度

低 - UI 整理

## 関連 Issue

- `20251129_03_remove-dashboard-header.md` - 同じくダッシュボード UI 整理
