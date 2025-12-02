# パッケージバージョン固定ガイド

## 概要

パッケージをインストールする際、デフォルトでは `^`（キャレット）や `~`（チルダ）付きのバージョン範囲が `package.json` に記録されます。これにより、意図しないバージョンアップが発生し、セキュリティリスクや互換性の問題が生じる可能性があります。

本ドキュメントでは、各パッケージマネージャでバージョンを固定する設定方法を解説します。

---

## バージョン範囲の記法

| 記法           | 例         | 意味                               |
| -------------- | ---------- | ---------------------------------- |
| 固定           | `4.17.21`  | 正確にこのバージョンのみ           |
| キャレット `^` | `^4.17.21` | メジャーバージョンを固定（4.x.x）  |
| チルダ `~`     | `~4.17.21` | メジャー・マイナーを固定（4.17.x） |

### なぜバージョンを固定すべきか

1. **再現性**: 全ての環境で同じバージョンを使用
2. **セキュリティ**: サプライチェーン攻撃で改ざんされた新バージョンの自動取得を防止
3. **安定性**: 意図しない破壊的変更を防止
4. **監査性**: 依存関係の変更を明確に追跡可能

---

## npm

### グローバル設定

```bash
npm config set save-exact true
```

これにより `~/.npmrc` に以下が追加されます：

```ini
save-exact=true
```

### プロジェクト単位の設定

プロジェクトルートに `.npmrc` を作成：

```ini
# .npmrc
save-exact=true
```

### 設定の確認

```bash
npm config get save-exact
# → true
```

### 参考

- [npm-config | npm Docs](https://docs.npmjs.com/cli/v10/commands/npm-config)
- [npm config: save-exact](https://docs.npmjs.com/cli/v10/using-npm/config#save-exact)

---

## Yarn Classic (v1)

### グローバル設定

```bash
yarn config set save-prefix ""
```

これにより `~/.yarnrc` に以下が追加されます：

```
save-prefix ""
```

**注意**: Yarn Classic には `save-exact` オプションがありません。代わりに `save-prefix` を空文字に設定することで、`^` や `~` なしの固定バージョンでインストールされます。

### プロジェクト単位の設定

プロジェクトルートに `.yarnrc` を作成：

```
# .yarnrc
save-prefix ""
```

### 設定の確認

```bash
yarn config get save-prefix
# → ""
```

### 参考

- [yarn config | Yarn Classic](https://classic.yarnpkg.com/en/docs/cli/config)

---

## Yarn Berry (v2/v3/v4)

### グローバル設定

```bash
yarn config set defaultSemverRangePrefix ""
```

これにより `~/.yarnrc.yml`（グローバル設定ファイル）に以下が追加されます：

```yaml
defaultSemverRangePrefix: ""
```

### プロジェクト単位の設定

プロジェクトルートの `.yarnrc.yml` に追加：

```yaml
# .yarnrc.yml
defaultSemverRangePrefix: ""
```

### 設定の確認

```bash
yarn config get defaultSemverRangePrefix
# → ""
```

### 参考

- [Yarn Configuration: defaultSemverRangePrefix | Yarn Berry](https://yarnpkg.com/configuration/yarnrc#defaultSemverRangePrefix)

---

## pnpm

### グローバル設定

```bash
pnpm config set save-exact true
```

これにより `~/.npmrc` に以下が追加されます：

```ini
save-exact=true
```

**注意**: pnpm は npm と同じ `.npmrc` ファイルを使用します。

### プロジェクト単位の設定

プロジェクトルートに `.npmrc` を作成：

```ini
# .npmrc
save-exact=true
```

### 設定の確認

```bash
pnpm config get save-exact
# → true
```

### 参考

- [pnpm config | pnpm](https://pnpm.io/cli/config)
- [.npmrc | pnpm](https://pnpm.io/npmrc#save-exact)

---

## Bun

### プロジェクト単位の設定

プロジェクトルートに `bunfig.toml` を作成：

```toml
# bunfig.toml
[install]
exact = true
```

### 参考

- [bunfig.toml | Bun Docs](https://bun.sh/docs/runtime/bunfig)

---

## 比較表

| 設定                     | npm                              | Yarn Classic (v1)                | Yarn Berry (v2+)                              | pnpm                              | Bun            |
| ------------------------ | -------------------------------- | -------------------------------- | --------------------------------------------- | --------------------------------- | -------------- |
| グローバル設定コマンド   | `npm config set save-exact true` | `yarn config set save-prefix ""` | `yarn config set defaultSemverRangePrefix ""` | `pnpm config set save-exact true` | -              |
| 設定ファイル             | `~/.npmrc`                       | `~/.yarnrc`                      | `~/.yarnrc.yml`                               | `~/.npmrc`                        | `bunfig.toml`  |
| プロジェクト設定ファイル | `.npmrc`                         | `.yarnrc`                        | `.yarnrc.yml`                                 | `.npmrc`                          | `bunfig.toml`  |
| 設定内容                 | `save-exact=true`                | `save-prefix ""`                 | `defaultSemverRangePrefix: ""`                | `save-exact=true`                 | `exact = true` |

---

## 動作確認

設定後にパッケージを追加すると、バージョンが固定されます：

```bash
# 設定前（デフォルト）
npm install lodash
# package.json → "lodash": "^4.17.21"

# 設定後
npm install lodash
# package.json → "lodash": "4.17.21"  ← ^ や ~ がつかない
```

---

## 既存プロジェクトへの適用

既存の `package.json` でバージョン範囲を使用している場合、以下の手順で固定バージョンに変換できます：

### 方法 1: 手動で編集

```json
// 変更前
{
  "dependencies": {
    "lodash": "^4.17.21",
    "express": "~4.18.2"
  }
}

// 変更後
{
  "dependencies": {
    "lodash": "4.17.21",
    "express": "4.18.2"
  }
}
```

### 方法 2: ロックファイルから現在のバージョンを取得

```bash
# npm
npm ls --depth=0

# Yarn
yarn list --depth=0

# pnpm
pnpm ls --depth=0
```

### 方法 3: npm-check-updates を使用

```bash
# インストール
npm install -g npm-check-updates

# 現在のバージョンに固定（^ や ~ を削除）
ncu --removeRange
```

---

## ベストプラクティス

### 1. 新規プロジェクトでは最初から固定

プロジェクト開始時に設定ファイルを作成：

```bash
# npm / pnpm
echo "save-exact=true" > .npmrc

# Yarn Classic
echo 'save-prefix ""' > .yarnrc

# Yarn Berry
echo 'defaultSemverRangePrefix: ""' > .yarnrc.yml
```

### 2. 設定ファイルをバージョン管理に含める

```bash
# .gitignore には含めない
git add .npmrc  # または .yarnrc, .yarnrc.yml
git commit -m "chore: enable exact version pinning"
```

### 3. 依存関係の更新は意図的に行う

```bash
# 特定のパッケージを更新
npm update lodash

# または最新バージョンに更新
npm install lodash@latest

# 変更をレビューしてコミット
git diff package.json package-lock.json
git add package.json package-lock.json
git commit -m "chore: update lodash to 4.17.22"
```

### 4. Renovate / Dependabot の活用

自動で依存関係を更新する PR を作成するツールを使用すると、セキュリティアップデートを見逃さずに対応できます：

- [Renovate](https://docs.renovatebot.com/)
- [Dependabot](https://docs.github.com/en/code-security/dependabot)

---

## 関連ドキュメント

- [NPM サプライチェーン攻撃対策ガイド](./NPM_SUPPLY_CHAIN_PROTECTION.md)
- [セキュアなパッケージインストールガイド](./SECURE_PACKAGE_INSTALL.md)

---

## 更新履歴

| 日付       | 内容     |
| ---------- | -------- |
| 2025-12-02 | 初版作成 |
