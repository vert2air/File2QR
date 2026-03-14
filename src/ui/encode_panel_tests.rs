#[cfg(test)]
mod tests {
    use crate::encode::EcLevel;
    use crate::ui::encode_panel::{EncodeMessage, EncodePanel, InputMode};

    // ── デフォルト状態 ────────────────────────────────────────

    #[test]
    fn test_default_state() {
        let panel = EncodePanel::default();
        assert_eq!(panel.input_mode, InputMode::File);
        assert_eq!(panel.file_path, "");
        assert_eq!(panel.direct_text, "");
        assert!(!panel.compress);
        assert_eq!(panel.ec_level, EcLevel::L);
        assert!(panel.qr_window.is_none());
        assert!(panel.error_msg.is_none());
    }

    // ── update() 経由の各メッセージ ───────────────────────────

    #[test]
    fn test_input_mode_change() {
        let mut panel = EncodePanel::default();
        assert_eq!(panel.input_mode, InputMode::File);

        panel.update(EncodeMessage::InputModeChanged(InputMode::DirectText));
        assert_eq!(panel.input_mode, InputMode::DirectText);

        panel.update(EncodeMessage::InputModeChanged(InputMode::File));
        assert_eq!(panel.input_mode, InputMode::File);
    }

    #[test]
    fn test_file_path_changed() {
        let mut panel = EncodePanel::default();
        panel.error_msg = Some("old error".to_string());

        panel.update(EncodeMessage::FilePathChanged("test.txt".to_string()));
        assert_eq!(panel.file_path, "test.txt");
        // エラーメッセージはクリアされる
        assert!(panel.error_msg.is_none());
    }

    #[test]
    fn test_file_dropped() {
        let mut panel = EncodePanel::default();
        panel.update(EncodeMessage::FileDropped("/tmp/foo.bin".to_string()));
        assert_eq!(panel.file_path, "/tmp/foo.bin");
        assert!(panel.error_msg.is_none());
    }

    #[test]
    fn test_direct_text_changed() {
        let mut panel = EncodePanel::default();
        panel.error_msg = Some("old error".to_string());

        panel.update(EncodeMessage::DirectTextChanged(
            "Hello, World!".to_string(),
        ));
        assert_eq!(panel.direct_text, "Hello, World!");
        assert!(panel.error_msg.is_none());
    }

    #[test]
    fn test_compress_toggle() {
        let mut panel = EncodePanel::default();
        assert!(!panel.compress);

        panel.update(EncodeMessage::CompressToggled(true));
        assert!(panel.compress);

        panel.update(EncodeMessage::CompressToggled(false));
        assert!(!panel.compress);
    }

    #[test]
    fn test_ec_level_change() {
        let mut panel = EncodePanel::default();
        assert_eq!(panel.ec_level, EcLevel::L);

        panel.update(EncodeMessage::EcLevelSelected(EcLevel::H));
        assert_eq!(panel.ec_level, EcLevel::H);

        panel.update(EncodeMessage::EcLevelSelected(EcLevel::Q));
        assert_eq!(panel.ec_level, EcLevel::Q);
    }

    // ── Generate: 空入力でのエラー ────────────────────────────

    #[test]
    fn test_generate_file_mode_empty_path() {
        let mut panel = EncodePanel::default();
        // input_mode = File, file_path = "" → エラー
        panel.update(EncodeMessage::GeneratePressed);
        assert!(panel.error_msg.is_some());
        assert!(panel.qr_window.is_none());
    }

    #[test]
    fn test_generate_text_mode_empty_text() {
        let mut panel = EncodePanel::default();
        panel.update(EncodeMessage::InputModeChanged(InputMode::DirectText));
        // direct_text = "" → エラー
        panel.update(EncodeMessage::GeneratePressed);
        assert!(panel.error_msg.is_some());
        assert!(panel.qr_window.is_none());
    }

    #[test]
    fn test_generate_file_not_found() {
        let mut panel = EncodePanel::default();
        panel.update(EncodeMessage::FilePathChanged(
            "/nonexistent/path/file.txt".to_string(),
        ));
        panel.update(EncodeMessage::GeneratePressed);
        assert!(panel.error_msg.is_some());
        assert!(panel.qr_window.is_none());
    }

    // ── Generate 成功: テキスト直接入力 ──────────────────────

    #[test]
    fn test_generate_text_mode_success() {
        let mut panel = EncodePanel::default();
        panel.update(EncodeMessage::InputModeChanged(InputMode::DirectText));
        panel.update(EncodeMessage::DirectTextChanged("Hello".to_string()));
        panel.update(EncodeMessage::GeneratePressed);

        assert!(panel.error_msg.is_none());
        assert!(panel.qr_window.is_some());

        let w = panel.qr_window.as_ref().unwrap();
        assert!(w.open);
        assert!(!w.fragments.is_empty());
    }

    // ── QrWindow 転送 ─────────────────────────────────────────

    #[test]
    fn test_qr_window_close_via_message() {
        let mut panel = EncodePanel::default();
        panel.update(EncodeMessage::InputModeChanged(InputMode::DirectText));
        panel.update(EncodeMessage::DirectTextChanged("Hi".to_string()));
        panel.update(EncodeMessage::GeneratePressed);
        assert!(panel.qr_window.is_some());

        // Close メッセージで qr_window が None になる
        panel.update(EncodeMessage::QrWindow(
            crate::ui::qr_window::QrWindowMessage::Close,
        ));
        assert!(panel.qr_window.is_none());
    }
}
