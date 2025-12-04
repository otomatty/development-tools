# Accordion Components Specification

## Related Files

- Implementation: `src/components/ui/accordion/mod.rs`
- Accordion: `src/components/ui/accordion/accordion.rs`

## Related Documentation

- Parent UI Module: `src/components/ui/mod.rs`

## Requirements

### 責務

Accordion コンポーネントモジュールは以下の責務を担当する：

1. **AccordionSection** - 折りたたみセクション

   - タイトルとオプションのアイコン
   - 展開/折りたたみアニメーション
   - キーボードアクセシビリティ（Enter, Space）
   - aria 属性による適切なアクセシビリティ

2. **Accordion** - アコーディオンコンテナ
   - 複数のセクションをグループ化
   - セクション間のギャップ設定

### 状態構造

#### AccordionItem

```rust
pub struct AccordionItem {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
}
```

#### AccordionSection Props

```rust
pub struct AccordionSectionProps {
    pub title: String,
    pub icon: Option<&'static str>,
    pub expanded: Signal<bool>,
    pub on_toggle: F,
    pub children: Children,
    pub max_height: &'static str,  // default: "500px"
    pub class: &'static str,
}
```

### 公開 API

```rust
pub use accordion::{Accordion, AccordionItem, AccordionSection};
```

### スタイリング仕様

- 角丸: `rounded-2xl`
- 背景: `bg-gm-bg-card/80 backdrop-blur-sm`
- ボーダー: `border border-gm-accent-cyan/20`
- ホバー: `hover:border-gm-accent-cyan/40 hover:shadow-gm-accent-cyan/10`
- トランジション: `transition-all duration-300`

#### ヘッダー

- パディング: `px-6 py-4`
- アイコン: `text-gm-accent-cyan`
- タイトル: `text-lg font-gaming font-bold text-white`
- 矢印アニメーション: `transition-transform duration-300 ease-in-out`

#### コンテンツ

- パディング: `px-6 pb-6 pt-2`
- 展開アニメーション: `max-height` と `opacity` のトランジション

## Test Cases

### TC-001: AccordionSection 展開/折りたたみ

- **Given**: AccordionSection が expanded=false で初期化
- **When**: ヘッダーがクリックされる
- **Then**: on_toggle コールバックが実行される

### TC-002: AccordionSection キーボード操作

- **Given**: AccordionSection ヘッダーにフォーカス
- **When**: Enter または Space キーが押される
- **Then**: on_toggle コールバックが実行される

### TC-003: AccordionSection アイコン表示

- **Given**: AccordionSection が icon="settings" で初期化
- **When**: レンダリングされる
- **Then**: settings アイコンがタイトルの前に表示される

### TC-004: AccordionSection aria 属性

- **Given**: AccordionSection が初期化
- **When**: レンダリングされる
- **Then**: aria-expanded, aria-controls, aria-labelledby が適切に設定される

### TC-005: Accordion ギャップ設定

- **Given**: Accordion が gap="gap-6" で初期化
- **When**: レンダリングされる
- **Then**: セクション間に gap-6 のスペースが適用される
