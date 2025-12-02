# セキュアなパッケージインストールガイド

## 概要

本ドキュメントでは、CI/CD 環境や本番デプロイにおいて、より安全にパッケージをインストールするためのコマンドと設定について解説します。

---

## npm ci と npm install の違い

`npm ci` は CI/CD 環境向けに設計された、より厳格なインストールコマンドです。

### 比較表

| 項目                     | npm ci                                            | npm install        |
| ------------------------ | ------------------------------------------------- | ------------------ |
| **ロックファイル**       | `package-lock.json` が必須、なければエラー        | なければ自動生成   |
| **ロックファイルの更新** | 更新しない（読み取り専用）                        | 必要に応じて更新   |
| **node_modules**         | 毎回完全に削除して再作成                          | 差分更新           |
| **バージョン整合性**     | `package.json` とロックファイルが不一致ならエラー | 自動で解決を試みる |
| **速度**                 | クリーンインストールは高速                        | 差分更新は高速     |
| **用途**                 | CI/CD、本番デプロイ                               | ローカル開発       |

### npm ci がセキュリティ上優れている理由

1. **再現性の保証**: ロックファイルに記載された正確なバージョンのみインストール
2. **改ざん検知**: ロックファイルと `package.json` の不一致を検出してエラー
3. **クリーン環境**: 既存の `node_modules` を削除するため、汚染されたモジュールが残らない
4. **ロックファイル保護**: ロックファイルを変更しないため、攻撃者による依存関係の改ざんを防止

### 参考

- [npm-ci | npm Docs](https://docs.npmjs.com/cli/v10/commands/npm-ci)
- [npm-install | npm Docs](https://docs.npmjs.com/cli/v10/commands/npm-install)

---

## Yarn での同等コマンド

### Yarn Classic (v1)

```bash
# npm ci に相当するコマンド
yarn install --frozen-lockfile
```

#### オプション一覧

| オプション          | 説明                                                         |
| ------------------- | ------------------------------------------------------------ |
| `--frozen-lockfile` | `yarn.lock` を更新せず、不一致があればエラー                 |
| `--ignore-scripts`  | ライフサイクルスクリプトを無効化                             |
| `--offline`         | ネットワークアクセスなしでインストール（キャッシュのみ使用） |
| `--non-interactive` | インタラクティブなプロンプトを無効化                         |

#### 推奨コマンド（CI/CD 向け）

```bash
yarn install --frozen-lockfile --ignore-scripts --non-interactive
```

#### 参考

- [yarn install | Yarn Classic](https://classic.yarnpkg.com/en/docs/cli/install)
- [yarn install --frozen-lockfile | Yarn Classic](https://classic.yarnpkg.com/en/docs/cli/install#toc-yarn-install-frozen-lockfile)

---

### Yarn Berry (v2/v3/v4)

```bash
# npm ci に相当するコマンド
yarn install --immutable
```

#### オプション一覧

| オプション          | 説明                                         |
| ------------------- | -------------------------------------------- |
| `--immutable`       | `yarn.lock` を更新せず、不一致があればエラー |
| `--immutable-cache` | キャッシュも変更不可（Yarn 3.1+）            |
| `--mode=skip-build` | ビルドスクリプトをスキップ                   |
| `--inline-builds`   | ビルド出力をインラインで表示                 |

#### 推奨コマンド（CI/CD 向け）

```bash
yarn install --immutable --mode=skip-build
```

#### 環境変数による設定

CI 環境では環境変数を使用して自動的に `--immutable` を有効化できます：

```bash
# CI 環境で自動的に immutable モードを有効化
YARN_ENABLE_IMMUTABLE_INSTALLS=true yarn install
```

#### 参考

- [yarn install | Yarn Berry](https://yarnpkg.com/cli/install)
- [Yarn Configuration: enableImmutableInstalls](https://yarnpkg.com/configuration/yarnrc#enableImmutableInstalls)

---

## pnpm での同等コマンド

```bash
# npm ci に相当するコマンド
pnpm install --frozen-lockfile
```

#### オプション一覧

| オプション          | 説明                                              |
| ------------------- | ------------------------------------------------- |
| `--frozen-lockfile` | `pnpm-lock.yaml` を更新せず、不一致があればエラー |
| `--ignore-scripts`  | ライフサイクルスクリプトを無効化                  |
| `--offline`         | ネットワークアクセスなしでインストール            |

#### 推奨コマンド（CI/CD 向け）

```bash
pnpm install --frozen-lockfile --ignore-scripts
```

#### 参考

- [pnpm install | pnpm](https://pnpm.io/cli/install)
- [pnpm install --frozen-lockfile | pnpm](https://pnpm.io/cli/install#--frozen-lockfile)

---

## Bun での同等コマンド

```bash
# npm ci に相当するコマンド
bun install --frozen-lockfile
```

#### オプション一覧

| オプション          | 説明                                         |
| ------------------- | -------------------------------------------- |
| `--frozen-lockfile` | `bun.lockb` を更新せず、不一致があればエラー |
| `--ignore-scripts`  | ライフサイクルスクリプトを無効化             |

#### 推奨コマンド（CI/CD 向け）

```bash
bun install --frozen-lockfile --ignore-scripts
```

#### 参考

- [bun install | Bun Docs](https://bun.sh/docs/cli/install)

---

## 機能比較表

| 機能                       | npm ci             | yarn v1 --frozen-lockfile | yarn v2+ --immutable | pnpm --frozen-lockfile | bun --frozen-lockfile |
| -------------------------- | ------------------ | ------------------------- | -------------------- | ---------------------- | --------------------- |
| ロックファイル必須         | ✅                 | ✅                        | ✅                   | ✅                     | ✅                    |
| ロックファイル更新禁止     | ✅                 | ✅                        | ✅                   | ✅                     | ✅                    |
| 不一致時エラー             | ✅                 | ✅                        | ✅                   | ✅                     | ✅                    |
| node_modules 自動削除      | ✅                 | ❌                        | ❌                   | ❌                     | ❌                    |
| スクリプト無効化オプション | `--ignore-scripts` | `--ignore-scripts`        | `--mode=skip-build`  | `--ignore-scripts`     | `--ignore-scripts`    |

---

## CI/CD 環境での使用例

### GitHub Actions

```yaml
name: CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm" # または 'yarn', 'pnpm'

      # npm の場合
      - name: Install dependencies (npm)
        run: npm ci --ignore-scripts

      # Yarn Classic (v1) の場合
      # - name: Install dependencies (Yarn v1)
      #   run: yarn install --frozen-lockfile --ignore-scripts

      # Yarn Berry (v2+) の場合
      # - name: Install dependencies (Yarn Berry)
      #   run: yarn install --immutable --mode=skip-build

      # pnpm の場合
      # - name: Install dependencies (pnpm)
      #   run: pnpm install --frozen-lockfile --ignore-scripts

      # 必要なビルドスクリプトを手動で実行
      - name: Run necessary build scripts
        run: |
          # 例: ネイティブモジュールのビルド
          # cd node_modules/bcrypt && npm run install
```

### GitLab CI

```yaml
stages:
  - build

build:
  stage: build
  image: node:20
  script:
    # npm の場合
    - npm ci --ignore-scripts

    # Yarn の場合
    # - yarn install --frozen-lockfile --ignore-scripts
  cache:
    paths:
      - node_modules/
```

### CircleCI

```yaml
version: 2.1

jobs:
  build:
    docker:
      - image: cimg/node:20.0
    steps:
      - checkout
      - restore_cache:
          keys:
            - v1-dependencies-{{ checksum "package-lock.json" }}
      - run:
          name: Install dependencies
          command: npm ci --ignore-scripts
      - save_cache:
          paths:
            - node_modules
          key: v1-dependencies-{{ checksum "package-lock.json" }}
```

---

## node_modules の完全削除が必要な場合

Yarn、pnpm、Bun は `npm ci` のように `node_modules` を自動削除しません。完全なクリーンインストールが必要な場合は手動で削除します：

```bash
# Yarn Classic
rm -rf node_modules && yarn install --frozen-lockfile --ignore-scripts

# Yarn Berry
rm -rf node_modules .yarn/cache && yarn install --immutable --mode=skip-build

# pnpm
rm -rf node_modules && pnpm install --frozen-lockfile --ignore-scripts

# Bun
rm -rf node_modules && bun install --frozen-lockfile --ignore-scripts
```

---

## ベストプラクティス

### 1. ロックファイルを必ずコミットする

```bash
# .gitignore に含めない
# package-lock.json  ← コメントアウトまたは削除
# yarn.lock          ← コメントアウトまたは削除
# pnpm-lock.yaml     ← コメントアウトまたは削除
```

### 2. CI 環境では厳格なインストールを使用する

| 環境         | 推奨コマンド                                                                  |
| ------------ | ----------------------------------------------------------------------------- |
| ローカル開発 | `npm install` / `yarn install`                                                |
| CI/CD        | `npm ci --ignore-scripts` / `yarn install --frozen-lockfile --ignore-scripts` |
| 本番デプロイ | `npm ci --ignore-scripts --omit=dev`                                          |

### 3. 依存関係の更新は意図的に行う

```bash
# 依存関係の更新（ローカル開発環境で実行）
npm update
# または
yarn upgrade

# 更新後、ロックファイルの変更をレビューしてコミット
git diff package-lock.json
git add package-lock.json
git commit -m "chore: update dependencies"
```

---

## 参考リンクまとめ

### npm

- [npm-ci | npm Docs](https://docs.npmjs.com/cli/v10/commands/npm-ci)
- [npm-install | npm Docs](https://docs.npmjs.com/cli/v10/commands/npm-install)

### Yarn Classic (v1)

- [yarn install | Yarn Classic](https://classic.yarnpkg.com/en/docs/cli/install)

### Yarn Berry (v2/v3/v4)

- [yarn install | Yarn Berry](https://yarnpkg.com/cli/install)
- [Yarn Configuration | Yarn Berry](https://yarnpkg.com/configuration/yarnrc)

### pnpm

- [pnpm install | pnpm](https://pnpm.io/cli/install)

### Bun

- [bun install | Bun Docs](https://bun.sh/docs/cli/install)

---

## 更新履歴

| 日付       | 内容     |
| ---------- | -------- |
| 2025-12-02 | 初版作成 |
