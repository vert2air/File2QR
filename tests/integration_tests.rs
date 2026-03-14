//! 結合テスト: encode → decode のエンドツーエンド検証
//!
//! ビジネスロジック層（encode/decode）と UI 層の公開 API を
//! 組み合わせて動作確認する。

use file2qr::decode;
use file2qr::encode::{self, EcLevel, EncodeInput};
use file2qr::ui::decode_panel::{DecodeMessage, DecodePanel};
use file2qr::ui::encode_panel::{EncodeMessage, EncodePanel, InputMode};
use file2qr::ui::qr_window::QrWindowMessage;

// ── encode → decode ラウンドトリップ ─────────────────────────────────────

/// テキストデータを encode して decode し、元データと一致することを確認
#[test]
fn test_roundtrip_text() {
    let data = b"Hello, integration test!".to_vec();
    let result = encode::encode(EncodeInput {
        data: data.clone(),
        filename: "hello.txt".to_string(),
        compress: false,
        ec_level: EcLevel::M,
    })
    .expect("encode failed");

    let lines: Vec<&str> =
        result.fragments.iter().map(|s| s.as_str()).collect();
    let entries = decode::parse_lines(&lines);
    let entry = entries.values().next().expect("no entry");

    assert!(entry.is_complete());
    let decoded = decode::reconstruct(entry).expect("reconstruct failed");
    assert_eq!(decoded, data);
}

/// バイナリデータのラウンドトリップ
#[test]
fn test_roundtrip_binary() {
    let data: Vec<u8> = (0u8..=255).collect();
    let result = encode::encode(EncodeInput {
        data: data.clone(),
        filename: "binary.bin".to_string(),
        compress: false,
        ec_level: EcLevel::L,
    })
    .expect("encode failed");

    let lines: Vec<&str> =
        result.fragments.iter().map(|s| s.as_str()).collect();
    let entries = decode::parse_lines(&lines);
    let entry = entries.values().next().expect("no entry");

    let decoded = decode::reconstruct(entry).expect("reconstruct failed");
    assert_eq!(decoded, data);
}

/// 圧縮ありのラウンドトリップ
#[test]
fn test_roundtrip_compressed() {
    let data = b"AAAAAAAAAA".repeat(500);
    let result = encode::encode(EncodeInput {
        data: data.clone(),
        filename: "repeat.bin".to_string(),
        compress: true,
        ec_level: EcLevel::L,
    })
    .expect("encode failed");

    let lines: Vec<&str> =
        result.fragments.iter().map(|s| s.as_str()).collect();
    let entries = decode::parse_lines(&lines);
    let entry = entries.values().next().expect("no entry");

    assert_eq!(entry.compressed, Some(true));
    let decoded = decode::reconstruct(entry).expect("reconstruct failed");
    assert_eq!(decoded, data);
}

/// 複数 QR に分割されるデータのラウンドトリップ
#[test]
fn test_roundtrip_multi_fragment() {
    let data = b"Z".repeat(8000);
    let result = encode::encode(EncodeInput {
        data: data.clone(),
        filename: "large.dat".to_string(),
        compress: false,
        ec_level: EcLevel::M,
    })
    .expect("encode failed");

    assert!(
        result.fragments.len() > 1,
        "8000 bytes should produce multiple fragments"
    );

    let lines: Vec<&str> =
        result.fragments.iter().map(|s| s.as_str()).collect();
    let entries = decode::parse_lines(&lines);
    let entry = entries.values().next().expect("no entry");

    assert!(entry.is_complete());
    let decoded = decode::reconstruct(entry).expect("reconstruct failed");
    assert_eq!(decoded, data);
}

/// 全エラー訂正レベルでラウンドトリップ
#[test]
fn test_roundtrip_all_ec_levels() {
    let data = b"EC level test data".to_vec();
    for &ec in EcLevel::all() {
        let result = encode::encode(EncodeInput {
            data: data.clone(),
            filename: "ec.txt".to_string(),
            compress: false,
            ec_level: ec,
        })
        .unwrap_or_else(|e| panic!("encode failed for {:?}: {}", ec, e));

        let lines: Vec<&str> =
            result.fragments.iter().map(|s| s.as_str()).collect();
        let entries = decode::parse_lines(&lines);
        let entry = entries.values().next().expect("no entry");
        let decoded = decode::reconstruct(entry).unwrap_or_else(|e| {
            panic!("reconstruct failed for {:?}: {}", ec, e)
        });
        assert_eq!(decoded, data);
    }
}

/// direct_text ファイル名のラウンドトリップ
#[test]
fn test_roundtrip_direct_text_filename() {
    let data = b"Direct text content".to_vec();
    let result = encode::encode(EncodeInput {
        data: data.clone(),
        filename: "(direct_text)".to_string(),
        compress: false,
        ec_level: EcLevel::L,
    })
    .expect("encode failed");

    let lines: Vec<&str> =
        result.fragments.iter().map(|s| s.as_str()).collect();
    let entries = decode::parse_lines(&lines);
    let entry = entries.values().next().expect("no entry");

    assert_eq!(entry.filename.as_deref(), Some("(direct_text)"));
    let decoded = decode::reconstruct(entry).expect("reconstruct failed");
    assert_eq!(decoded, data);
}

/// 特殊文字を含むファイル名のラウンドトリップ
#[test]
fn test_roundtrip_special_chars_in_filename() {
    let data = b"data".to_vec();
    let filename = "my file (v2) [draft].txt";
    let result = encode::encode(EncodeInput {
        data: data.clone(),
        filename: filename.to_string(),
        compress: false,
        ec_level: EcLevel::M,
    })
    .expect("encode failed");

    let lines: Vec<&str> =
        result.fragments.iter().map(|s| s.as_str()).collect();
    let entries = decode::parse_lines(&lines);
    let entry = entries.values().next().expect("no entry");

    assert_eq!(entry.filename.as_deref(), Some(filename));
    let decoded = decode::reconstruct(entry).expect("reconstruct failed");
    assert_eq!(decoded, data);
}

// ── フラグメント欠損時の挙動 ──────────────────────────────────────────────

/// フラグメントが欠損していると is_complete() が false になる
#[test]
fn test_incomplete_fragments_not_reconstructable() {
    let data = b"W".repeat(6000);
    let result = encode::encode(EncodeInput {
        data: data.clone(),
        filename: "incomplete.dat".to_string(),
        compress: false,
        ec_level: EcLevel::M,
    })
    .expect("encode failed");

    assert!(result.fragments.len() > 2, "need at least 3 fragments");

    // 最後のフラグメントを除いて parse
    let partial: Vec<&str> = result.fragments[..result.fragments.len() - 1]
        .iter()
        .map(|s| s.as_str())
        .collect();
    let entries = decode::parse_lines(&partial);
    let entry = entries.values().next().expect("no entry");

    assert!(!entry.is_complete());
    assert!(!entry.missing_indices().is_empty());
    assert!(decode::reconstruct(entry).is_err());
}

// ── EncodePanel の UI 統合 ─────────────────────────────────────────────────

/// EncodePanel でテキスト生成 → QrWindow が作られ、ページ操作できる
#[test]
fn test_encode_panel_generates_qr_window() {
    let mut panel = EncodePanel::default();
    panel.update(EncodeMessage::InputModeChanged(InputMode::DirectText));
    panel.update(EncodeMessage::DirectTextChanged(
        "Hello from panel".to_string(),
    ));
    panel.update(EncodeMessage::GeneratePressed);

    let w = panel.qr_window.as_ref().expect("qr_window should be Some");
    assert!(w.open);
    assert!(!w.fragments.is_empty());
    assert_eq!(w.page, 0);
}

/// Close メッセージで QrWindow が閉じる
#[test]
fn test_encode_panel_close_qr_window() {
    let mut panel = EncodePanel::default();
    panel.update(EncodeMessage::InputModeChanged(InputMode::DirectText));
    panel.update(EncodeMessage::DirectTextChanged("test".to_string()));
    panel.update(EncodeMessage::GeneratePressed);
    assert!(panel.qr_window.is_some());

    panel.update(EncodeMessage::QrWindow(QrWindowMessage::Close));
    assert!(panel.qr_window.is_none());
}

/// 大量フラグメントでページ移動ができる
#[test]
fn test_encode_panel_multipage_navigation() {
    let mut panel = EncodePanel::default();
    panel.update(EncodeMessage::InputModeChanged(InputMode::DirectText));
    // 十分大きなテキストで複数QRコードを生成
    let large_text = "A".repeat(5000);
    panel.update(EncodeMessage::DirectTextChanged(large_text));
    panel.update(EncodeMessage::GeneratePressed);

    let w = panel.qr_window.as_ref().expect("qr_window should be Some");
    let total_pages = w.total_pages();

    if total_pages > 1 {
        // NextPage で進める
        panel.update(EncodeMessage::QrWindow(QrWindowMessage::NextPage));
        assert_eq!(panel.qr_window.as_ref().unwrap().page, 1);

        // PrevPage で戻れる
        panel.update(EncodeMessage::QrWindow(QrWindowMessage::PrevPage));
        assert_eq!(panel.qr_window.as_ref().unwrap().page, 0);
    }
}

// ── DecodePanel の UI 統合 ────────────────────────────────────────────────

/// 存在しないファイルを追加するとエラーメッセージが出る
#[test]
fn test_decode_panel_nonexistent_file_error() {
    let mut panel = DecodePanel::default();
    panel.update(DecodeMessage::FileDropped(
        "/nonexistent/path/file.txt".to_string(),
    ));
    // ファイルはリストに追加されるがエラーメッセージが設定される
    assert_eq!(panel.input_files.len(), 1);
    assert!(panel.error_msg.is_some());
}

/// 同じファイルを2回追加しても重複しない
#[test]
fn test_decode_panel_no_duplicate_files() {
    let mut panel = DecodePanel::default();
    panel.update(DecodeMessage::FileDropped("dup.txt".to_string()));
    panel.update(DecodeMessage::FileDropped("dup.txt".to_string()));
    assert_eq!(panel.input_files.len(), 1);
}

/// ハッシュ選択なしでデコードボタンを押しても何も起きない
#[test]
fn test_decode_panel_decode_without_selection() {
    let mut panel = DecodePanel::default();
    panel.update(DecodeMessage::DecodePressed);
    // selected_hashes が空なのでエラーも成功もない
    assert!(panel.status_msg.is_none());
    assert!(panel.error_msg.is_none());
}

// ── encode → DecodePanel 統合フロー ──────────────────────────────────────

/// encode で生成したフラグメントを一時ファイルに書き、
/// DecodePanel で読み込んで正しく解析できることを確認
#[test]
fn test_encode_then_decode_panel_via_file() {
    use std::io::Write;

    let data = b"Integration via file".to_vec();
    let result = encode::encode(EncodeInput {
        data: data.clone(),
        filename: "via_file.txt".to_string(),
        compress: false,
        ec_level: EcLevel::L,
    })
    .expect("encode failed");

    // フラグメントを一時ファイルに書き出す
    let tmp = std::env::temp_dir().join("file2qr_integration_test.txt");
    {
        let mut f = std::fs::File::create(&tmp).expect("create tmp file");
        for frag in &result.fragments {
            writeln!(f, "{}", frag).expect("write frag");
        }
    }

    // DecodePanel でファイルを読み込む
    let mut panel = DecodePanel::default();
    panel
        .update(DecodeMessage::FileDropped(tmp.to_string_lossy().to_string()));

    assert_eq!(panel.input_files.len(), 1);
    assert!(panel.error_msg.is_none(), "should parse without error");
    assert!(!panel.entries.is_empty(), "entries should be populated");

    // エントリが完全であることを確認
    let entry = panel.entries.values().next().expect("no entry");
    assert!(entry.is_complete());

    // 後片付け
    let _ = std::fs::remove_file(&tmp);
}
