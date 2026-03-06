# File2QR

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
