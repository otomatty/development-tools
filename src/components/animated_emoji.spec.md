# Animated Emoji Component Specification

## Related Files

- Implementation: `src/components/animated_emoji.rs`
- CSS Animations: `input.css` (Animated Emoji CSS section)
- Tests: `src/components/animated_emoji.rs` (module tests)

## Related Documentation

- Issue: [GitHub Issue #40](https://github.com/otomatty/development-tools/issues/40)
- Related Issue: [GitHub Issue #45](https://github.com/otomatty/development-tools/issues/45) (ホバー時のみアニメーション)

## Requirements

### 責務

- 特定の絵文字にアニメーションを適用
- AnimationContext の設定を尊重
- ホバー時のみアニメーションするモードをサポート
- ストリーク値などに基づいたアニメーション強度の動的変更

### 対応絵文字

| 絵文字 | タイプ   | 用途                      | アニメーション         |
| ------ | -------- | ------------------------- | ---------------------- |
| 🔥     | Fire     | ストリーク表示            | flame (揺れる炎)       |
| 🏆     | Trophy   | 最高記録、バッジ          | shine (輝き)           |
| ⭐     | Star     | スター数、評価            | twinkle (瞬き)         |
| 🎯     | Target   | 目標達成、バッジ          | pulse-scale (拡大縮小) |
| 💪     | Muscle   | ストリークマイルストーン  | flex (力こぶ)          |
| 👑     | Crown    | 最高レベルバッジ          | float (浮遊)           |
| 🎉     | Party    | レベルアップ、バッジ獲得  | bounce (バウンス)      |
| ✨     | Sparkles | クオリティバッジ、XP 通知 | sparkle (キラキラ)     |
| 🚀     | Rocket   | 成長・進捗                | launch (発射)          |

### コンポーネント

#### AnimatedEmoji

基本的なアニメーション絵文字コンポーネント。

```rust
#[component]
pub fn AnimatedEmoji(
    emoji: EmojiType,
    #[prop(default = "text-2xl")] size: &'static str,
    #[prop(default = false)] hover_only: bool,
    #[prop(default = AnimationIntensity::Normal)] intensity: AnimationIntensity,
    #[prop(default = "")] class: &'static str,
) -> impl IntoView
```

#### AnimatedEmojiWithIntensity

値に基づいてアニメーション強度が動的に変化するコンポーネント。

```rust
#[component]
pub fn AnimatedEmojiWithIntensity(
    emoji: EmojiType,
    #[prop(default = "text-2xl")] size: &'static str,
    #[prop(default = false)] hover_only: bool,
    #[prop(into)] value: Signal<i32>,
    #[prop(default = [1, 7, 30])] thresholds: [i32; 3],
    #[prop(default = "")] class: &'static str,
) -> impl IntoView
```

### アニメーション強度

| 強度   | 説明                   | CSS 修飾子                    |
| ------ | ---------------------- | ----------------------------- |
| None   | アニメーションなし     | -                             |
| Subtle | 控えめなアニメーション | animation-subtle (3 秒周期)   |
| Normal | 標準のアニメーション   | -                             |
| Strong | 強いアニメーション     | animation-strong (0.5 秒周期) |

### AnimationContext との連携

- `AnimationContext.enabled`が`false`の場合、アニメーションは無効
- `use_animation_context_or_default()`で取得
- 個別のコンポーネントで`hover_only`を設定可能

## Test Cases

### TC-001: 基本表示

- Given: AnimatedEmoji コンポーネント
- When: EmojiType::Fire を指定して表示
- Then: 🔥 が表示される

### TC-002: アニメーション有効時

- Given: AnimationContext が有効
- When: AnimatedEmoji を表示
- Then: CSS アニメーションクラスが適用される

### TC-003: アニメーション無効時

- Given: AnimationContext が無効
- When: AnimatedEmoji を表示
- Then: CSS アニメーションクラスが適用されない

### TC-004: ホバー時のみアニメーション

- Given: hover_only=true
- When: ホバーしていない状態
- Then: アニメーションなし

### TC-005: ホバー時のみアニメーション（ホバー時）

- Given: hover_only=true
- When: ホバーした状態
- Then: アニメーションが有効

### TC-006: 強度ベースのアニメーション

- Given: AnimatedEmojiWithIntensity で value=35, thresholds=[1,7,30]
- When: 表示
- Then: AnimationIntensity::Strong が適用される

### TC-007: 強度ベースのアニメーション（低値）

- Given: AnimatedEmojiWithIntensity で value=3, thresholds=[1,7,30]
- When: 表示
- Then: AnimationIntensity::Subtle が適用される

## Implementation Notes

### フェーズ 1（現在の実装）

- CSS アニメーションによる実装
- Google Noto Animated Emoji のコンセプトを参考にしたアニメーション
- 軽量で追加の依存関係なし

### フェーズ 2（将来の拡張）

- Lottie 統合による高品質アニメーション
- Google Noto Animated Emoji の Lottie ファイル使用
- `public/assets/emoji/`にアセットをバンドル

## 使用箇所

- `src/components/home/stats_display.rs` - ストリーク表示（🔥🏆）
- `src/components/home/xp_notification.rs` - XP 通知（✨）
