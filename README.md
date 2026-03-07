# File2QR

[![CI](https://github.com/vert2air/File2QR/actions/workflows/ci.yml/badge.svg)](https://github.com/vert2air/File2QR/actions/workflows/ci.yml)
[![Release](https://github.com/vert2air/File2QR/actions/workflows/release.yml/badge.svg)](https://github.com/vert2air/File2QR/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

任意のファイルをQRコードに分割して表示・復元するツール

## 特徴

### QRコード生成
  - L: 2953 byte/QR
  - M: 2331 byte/QR
  - Q: 1663 byte/QR
  - H: 1272 byte/QR
- xz圧縮オプション対応
- エラー訂正レベル選択

### データ復元
- 不足フラグメント表示
- 出力先ディレクトリ選択
- 復元後に出力フォルダを開く機能

### UI改善
- 行数・列数ボタン
- 全画面表示
- デフォルトscale: 2倍

## ビルド

```bash
cargo build --release
```

## 実行

```bash
cargo run --release
```

## 環境変数

```bash
# QRコード拡大率（デフォルト: 2）
FILE2QR_SCALE=5 cargo run
```

## ライセンス

MIT License

## 仕様

詳細は `docs/spec.md` を参照してください。

---

## 開発者向け情報

### CI/CD

このプロジェクトはGitHub Actionsを使用しています：

- **CI**: プッシュ/PR時に自動テスト・ビルド（Linux/Windows/macOS）
- **Release**: タグプッシュ時に自動リリース作成

### リリース方法

```bash
# バージョンタグを作成してプッシュ
git tag v0.1.0
git push origin v0.1.0

# GitHub Actionsが自動的に:
# 1. Linux/Windows/macOS用バイナリをビルド
# 2. GitHubリリースを作成
# 3. バイナリをアップロード
```

### コードフォーマット

```bash
# フォーマット確認
cargo fmt -- --check

# 自動フォーマット
cargo fmt

# Linter実行
cargo clippy -- -D warnings
```

### 依存関係の更新

Dependabotが週次で自動的にPRを作成します。

手動更新:
```bash
cargo update
```
