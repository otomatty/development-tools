# TODO Collector

コードベース内の TODO/FIXME/HACK/XXX/NOTE コメントを収集し、一覧化するCLIツールです。

## 機能

- 指定ディレクトリ内のファイルを再帰的にスキャン
- TODO, FIXME, HACK, XXX, NOTE コメントを検出
- ファイルパス、行番号、コメント内容を出力
- テキストおよびJSON形式での出力に対応
- 優先度によるソート機能

## インストール

```bash
cd tools/todo-collector
cargo build --release
```

## 使用方法

### 基本的な使い方

```bash
# カレントディレクトリをスキャン
./target/release/todo-collector

# 特定のディレクトリをスキャン
./target/release/todo-collector --scan-dir ~/projects/myapp
```

### オプション

| オプション | 短縮形 | 説明 |
|-----------|--------|------|
| `--scan-dir` | `-s` | スキャン対象ディレクトリ |
| `--pattern` | `-p` | 検出パターン（カンマ区切り、デフォルト: TODO,FIXME,HACK,XXX,NOTE） |
| `--exclude` | `-e` | 除外ディレクトリ（カンマ区切り） |
| `--output` | `-o` | 出力形式（text/json） |
| `--priority` | - | 優先度でソート（FIXME > TODO > HACK > XXX > NOTE） |

### 出力例

#### テキスト出力

```
╔═══════════════════════════════════════════════════════════════╗
║                      TODO Collector                           ║
╚═══════════════════════════════════════════════════════════════╝

📊 Total: 15

   FIXME:    5 █████
   TODO :    8 ████████
   HACK :    2 ██

📁 src/app.rs
  L  42 FIXME エラーハンドリングを追加する
  L  87 TODO  リファクタリング予定
```

#### JSON出力

```json
{
  "summary": {
    "total": 15,
    "by_type": {
      "TODO": 8,
      "FIXME": 5,
      "HACK": 2
    }
  },
  "items": [
    {
      "type": "FIXME",
      "file": "src/app.rs",
      "line": 42,
      "content": "エラーハンドリングを追加する"
    }
  ]
}
```

## 対応言語

以下のプログラミング言語のソースファイルをスキャンします：

- Rust, Go, C/C++, Java, Kotlin, Swift
- JavaScript, TypeScript, Vue, Svelte
- Python, Ruby, PHP, Lua
- HTML, CSS, SCSS
- SQL, Shell scripts
- その他多数

## GUI連携

development-tools アプリケーションのGUIから使用できます。
tool.json でGUI表示の設定が定義されています。

## ライセンス

MIT

