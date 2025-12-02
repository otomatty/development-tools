# NPM サプライチェーン攻撃対策ガイド

## 概要

本ドキュメントは、2024 年 11 月に報告された「Shai-hulud 2.0」などの NPM サプライチェーン攻撃に対する防御策をまとめたものです。

### 参考情報

- [Shai-hulud 2.0 キャンペーンがクラウドと開発者エコシステムを標的に - Trend Micro](https://www.trendmicro.com/ja_jp/research/25/k/shai-hulud-2-0-targets-cloud-and-developer-systems.html)

---

## 攻撃の仕組み

Shai-hulud 2.0 は以下の手順で攻撃を実行します：

1. **preinstall スクリプトの悪用**: `npm install` 時に自動実行される `preinstall` スクリプトにマルウェアを仕込む
2. **認証情報の窃取**: AWS、GCP、Azure、GitHub、NPM のトークンや認証情報を窃取
3. **自己増殖**: 被害者が管理する NPM パッケージにバックドアを挿入し、再公開
4. **破壊的行為**: 認証に失敗した場合、ユーザデータを消去する破壊的コマンドを実行

---

## 対策方法

### 1. パッケージインストール時のスクリプト実行を無効化

パッケージマネージャごとに `preinstall`、`postinstall` などのライフサイクルスクリプトを無効化できます。

#### npm

```bash
# グローバル設定（永続的）
npm config set ignore-scripts true

# 一時的にスクリプトを無効化してインストール
npm install --ignore-scripts
```

#### Yarn Classic (v1)

```bash
# グローバル設定
yarn config set ignore-scripts true

# 一時的にスクリプトを無効化してインストール
yarn install --ignore-scripts
```

プロジェクトルートに `.yarnrc` ファイルを作成して設定することも可能です：

```
# .yarnrc
ignore-scripts true
```

#### Yarn Berry (v2/v3/v4)

```bash
# グローバル設定
yarn config set enableScripts false

# 一時的にスクリプトを無効化してインストール
yarn install --mode=skip-build
```

プロジェクトルートに `.yarnrc.yml` ファイルを作成して設定することも可能です：

```yaml
# .yarnrc.yml
enableScripts: false
```

#### pnpm

```bash
# グローバル設定
pnpm config set ignore-scripts true

# 一時的にスクリプトを無効化してインストール
pnpm install --ignore-scripts
```

プロジェクトルートに `.npmrc` ファイルを作成して設定することも可能です：

```
# .npmrc
ignore-scripts=true
```

#### Bun

```bash
# 一時的にスクリプトを無効化してインストール
bun install --ignore-scripts
```

`bunfig.toml` で設定することも可能です：

```toml
# bunfig.toml
[install]
lifecycle-scripts = false
```

---

### 2. 注意事項

`ignore-scripts` を有効にすると、以下の影響があります：

| 影響                 | 説明                                                                        |
| -------------------- | --------------------------------------------------------------------------- |
| ネイティブモジュール | `node-gyp` を使用するパッケージ（`bcrypt`、`sharp` など）がビルドされない   |
| CLI ツール           | グローバルインストールする CLI ツールが正常にセットアップされない場合がある |
| 依存関係の初期化     | 一部のパッケージが必要な初期化処理を実行できない                            |

**対処法**: 必要なパッケージのスクリプトは個別に手動で実行します。

```bash
# 特定のパッケージのスクリプトを手動実行
cd node_modules/<package-name>
npm run postinstall
```

---

### 3. 追加の防御策

#### 3.1 脆弱性スキャンの定期実行

```bash
# npm
npm audit

# Yarn
yarn audit

# pnpm
pnpm audit
```

#### 3.2 ロックファイルの使用

ロックファイルを必ずバージョン管理に含め、依存関係のバージョンを固定します。

| パッケージマネージャ | ロックファイル      |
| -------------------- | ------------------- |
| npm                  | `package-lock.json` |
| Yarn                 | `yarn.lock`         |
| pnpm                 | `pnpm-lock.yaml`    |
| Bun                  | `bun.lockb`         |

#### 3.3 パッケージの信頼性確認

新しいパッケージを導入する前に以下を確認します：

- **ダウンロード数**: 週間ダウンロード数が極端に少ないパッケージは注意
- **メンテナンス状況**: 最終更新日、Issue/PR の対応状況
- **パッケージ名**: タイポスクワッティング（似た名前の悪意あるパッケージ）に注意
- **依存関係**: 不必要に多くの依存を持つパッケージは精査

#### 3.4 認証情報の保護

- **環境変数**: 認証情報は環境変数で管理し、コードにハードコードしない
- **`.npmrc` / `.yarnrc`**: 認証トークンが含まれるファイルは `.gitignore` に追加
- **シークレット管理**: CI/CD 環境では適切なシークレット管理サービスを使用

```bash
# .gitignore に追加
.npmrc
.yarnrc
.yarnrc.yml
```

#### 3.5 CI/CD 環境での対策

```yaml
# GitHub Actions の例
- name: Install dependencies (with ignore-scripts)
  run: npm ci --ignore-scripts

- name: Run necessary build scripts manually
  run: |
    cd node_modules/some-native-package
    npm run build
```

---

### 4. 推奨される設定（プロジェクト単位）

プロジェクトルートに以下の設定ファイルを作成し、チーム全体で統一した設定を使用します。

#### `.npmrc`（npm / pnpm 共通）

```ini
# スクリプト実行を無効化
ignore-scripts=true

# 厳格なバージョン管理
save-exact=true

# 脆弱性が見つかった場合にインストールを失敗させる
audit=true
```

#### `.yarnrc.yml`（Yarn Berry）

```yaml
# スクリプト実行を無効化
enableScripts: false

# 厳格な依存関係管理
nodeLinker: node-modules
```

---

## まとめ

| 対策                      | 効果                         | 実装難易度 |
| ------------------------- | ---------------------------- | ---------- |
| `ignore-scripts` の有効化 | preinstall 攻撃の防止        | 低         |
| ロックファイルの使用      | バージョン固定による攻撃防止 | 低         |
| 定期的な `audit` 実行     | 既知の脆弱性検出             | 低         |
| パッケージ導入前の確認    | 悪意あるパッケージの検出     | 中         |
| 認証情報の適切な管理      | 認証情報窃取の防止           | 中         |
| CI/CD での対策            | 自動化された防御             | 中         |

---

## 更新履歴

| 日付       | 内容     |
| ---------- | -------- |
| 2025-12-02 | 初版作成 |
