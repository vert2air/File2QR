#[cfg(test)]
mod tests {
    use crate::encode::EcLevel;
    use crate::ui::encode_panel::EncodePanel;

    #[test]
    fn test_encode_panel_default_state() {
        let panel = EncodePanel::default();
        
        assert_eq!(panel.file_path, "");
        assert_eq!(panel.direct_text, "");
        assert_eq!(panel.compress, false);
        assert_eq!(panel.ec_level, EcLevel::L);  // デフォルトはL
        assert!(panel.qr_window.is_none());
        assert!(panel.error_msg.is_none());
    }

    #[test]
    fn test_encode_panel_file_path_validation() {
        let mut panel = EncodePanel::default();
        
        // 空のファイルパスは無効
        panel.file_path = "".to_string();
        assert!(panel.file_path.is_empty());
        
        // ファイルパスが設定されている
        panel.file_path = "test.txt".to_string();
        assert_eq!(panel.file_path, "test.txt");
    }

    #[test]
    fn test_encode_panel_ec_level_change() {
        let mut panel = EncodePanel::default();
        
        // デフォルトはL
        assert_eq!(panel.ec_level, EcLevel::L);
        
        // 変更可能
        panel.ec_level = EcLevel::H;
        assert_eq!(panel.ec_level, EcLevel::H);
        
        panel.ec_level = EcLevel::L;
        assert_eq!(panel.ec_level, EcLevel::L);
    }

    #[test]
    fn test_encode_panel_compression_toggle() {
        let mut panel = EncodePanel::default();
        
        // デフォルトは圧縮なし
        assert_eq!(panel.compress, false);
        
        // トグル
        panel.compress = true;
        assert_eq!(panel.compress, true);
        
        panel.compress = false;
        assert_eq!(panel.compress, false);
    }

    #[test]
    fn test_encode_panel_error_handling() {
        let mut panel = EncodePanel::default();
        
        // エラーメッセージが設定できる
        panel.error_msg = Some("Test error".to_string());
        assert_eq!(panel.error_msg.as_ref().unwrap(), "Test error");
        
        // クリアできる
        panel.error_msg = None;
        assert!(panel.error_msg.is_none());
    }

    #[test]
    fn test_encode_panel_text_mode() {
        let mut panel = EncodePanel::default();
        
        // テキストモードのテスト
        panel.direct_text = "Hello, World!".to_string();
        assert_eq!(panel.direct_text, "Hello, World!");
        assert!(!panel.direct_text.is_empty());
    }
}
