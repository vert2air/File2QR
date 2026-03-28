# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-03-29

### Changed
- **QRコード容量を理論値最大まで拡張**
  - 5%の安全マージンを削除し、理論値通りの容量を使用

### Added
- **Windowsでのコンソールウィンドウ非表示**
  - 実行ファイルをダブルクリック起動時にコンソール画面が表示されない

### Details

#### QRコード容量の変更

| レベル | 変更前 | 変更後 | 増加量 | 増加率 |
|--------|--------|--------|--------|--------|
| L      | 2800   | 2953   | +153   | +5.5%  |
| M      | 2200   | 2331   | +131   | +6.0%  |
| Q      | 1580   | 1663   | +83    | +5.3%  |
| H      | 1210   | 1272   | +62    | +5.1%  |

#### 実質的な利用可能容量（qr_cap = max_bytes - 13）

| レベル | 変更前 | 変更後 | 増加量 |
|--------|--------|--------|--------|
| L      | 2787   | 2940   | +153   |
| M      | 2187   | 2318   | +131   |
| Q      | 1567   | 1650   | +83    |
| H      | 1197   | 1259   | +62    |

## [0.3.0] - 2026-03-25

### Changed
- **GUIフレームワークをeguiからicedに移行**
  - より安定したレンダリング
  - OpenGLサポートがない環境でも動作（tiny-skiaバックエンド）
  - 仮想環境での互換性向上
- 依存関係の更新
  - iced 0.13（tiny-skia、CPU レンダリング）
  - image: 0.25.9 → 0.25.10
  - rfd: 0.17.2 → 0.15（icedとの互換性のため）

### Added
- QRコード全画面表示機能
  - QRコードウィンドウを全画面で表示可能
  - 大画面での視認性向上

### Fixed
- QRコードのドット形状を正方形に近づけるよう改善
  - より読み取りやすいQRコード生成
- 矢印キーでQRグループが切り替わらない問題を修正
  - キーボードナビゲーションの改善
- 復元したテキストをクリップボードに貼り付けできない問題を修正
  - テキスト選択とコピー機能の安定性向上
- テストで見つかったバグを修正
  - テストコードの更新と改善

### Technical Details
- eframe/eguiからicedへの完全移行
  - 約2,800行の変更（追加/削除）
  - すべてのUIコンポーネントをiced形式に書き換え
  - テストコードも全面的に更新
- tiny-skiaレンダラーを採用
  - CPUベースのソフトウェアレンダリング
  - OpenGLやVulkanが不要
  - 仮想マシンや古いハードウェアでも確実に動作

## [0.2.0] - 2026-03-12

### Added
- テキストコピー機能
  - デコードパネルで復元されたテキストをマウスで範囲選択可能
  - Ctrl+C でクリップボードにコピー
  - 「全選択してコピー」ボタンでワンクリックコピー
- レンダラー自動フォールバック機能
  - wgpu で起動失敗時に自動的に glow レンダラーにフォールバック
  - 環境変数 `FILE2QR_RENDERER` で特定のレンダラーを指定可能
- 仮想環境（Windows 11）での動作サポート

### Changed
- デフォルトレンダラーを glow から wgpu に変更
  - より広い環境で動作するように変更
  - OpenGL 2.0 未満の環境でも DirectX 12/Vulkan で動作
- 依存関係の更新
  - rfd: 0.15.4 → 0.17.2
  - actions/checkout: v4 → v6
  - actions/cache: v4 → v5
  - actions/upload-artifact: v4 → v7

### Fixed
- ウィジェットID衝突による赤字エラーメッセージを修正
  - 「📂 選択...」ボタンのラベルを一意に変更
  - QRウィンドウの +/- ボタンに一意のIDを付与
  - ID衝突警告を無効化
- 起動時の不要なログ出力を抑制
  - Vulkan 初期化の警告を非表示
  - OpenGL バージョン警告を非表示
  - DirectX 12 シェーダーコンパイル詳細を非表示
  - egui_wgpu の詳細情報を非表示

## [0.1.0] - 2026-03-08

Initial release.

### Added
- ファイルをQRコードに分割して表示
- QRコードから元ファイルを復元
- xz圧縮オプション
- エラー訂正レベル選択（L/M/Q/H）
- QRコード表示ウィンドウ
  - グリッド表示（行・列調整可能）
  - ページネーション
  - 全画面表示（F11）
- 日本語フォント自動読み込み
- CI/CD（GitHub Actions）
- 包括的なテストスイート（48テスト）

[Unreleased]: https://github.com/vert2air/File2QR/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/vert2air/File2QR/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/vert2air/File2QR/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/vert2air/File2QR/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/vert2air/File2QR/releases/tag/v0.1.0
