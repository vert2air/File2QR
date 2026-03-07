#[cfg(test)]
mod tests {
    use crate::ui::decode_panel::{DecodePanel, OutputDir};

    #[test]
    fn test_decode_panel_default_state() {
        let panel = DecodePanel::default();

        assert!(panel.input_files.is_empty());
        assert_eq!(panel.file_path_input, "");
        assert!(panel.entries.is_empty());
        assert!(panel.selected_hashes.is_empty());
        assert!(matches!(panel.output_dir, OutputDir::SameAsInput)); // デフォルトはSameAsInput
        assert!(panel.decoded_text.is_none());
        assert!(panel.status_msg.is_none());
        assert!(panel.error_msg.is_none());
    }

    #[test]
    fn test_decode_panel_add_file() {
        let mut panel = DecodePanel::default();

        // ファイルパスを追加
        panel.input_files.push("test1.txt".to_string());
        assert_eq!(panel.input_files.len(), 1);

        panel.input_files.push("test2.txt".to_string());
        assert_eq!(panel.input_files.len(), 2);

        assert_eq!(panel.input_files[0], "test1.txt");
        assert_eq!(panel.input_files[1], "test2.txt");
    }

    #[test]
    fn test_decode_panel_remove_file() {
        let mut panel = DecodePanel::default();

        panel.input_files.push("test1.txt".to_string());
        panel.input_files.push("test2.txt".to_string());
        panel.input_files.push("test3.txt".to_string());

        // 2番目を削除
        panel.input_files.remove(1);
        assert_eq!(panel.input_files.len(), 2);
        assert_eq!(panel.input_files[0], "test1.txt");
        assert_eq!(panel.input_files[1], "test3.txt");
    }

    #[test]
    fn test_decode_panel_output_dir_selection() {
        let mut panel = DecodePanel::default();

        // デフォルトはSameAsInput
        assert!(matches!(panel.output_dir, OutputDir::SameAsInput));

        // 変更可能
        panel.output_dir = OutputDir::CurrentDir;
        assert!(matches!(panel.output_dir, OutputDir::CurrentDir));

        panel.output_dir = OutputDir::SameAsInput;
        assert!(matches!(panel.output_dir, OutputDir::SameAsInput));

        panel.output_dir = OutputDir::Custom("/custom/path".to_string());
        if let OutputDir::Custom(path) = &panel.output_dir {
            assert_eq!(path, "/custom/path");
        } else {
            panic!("Expected Custom output dir");
        }
    }

    #[test]
    fn test_decode_panel_hash_selection() {
        let mut panel = DecodePanel::default();

        // ハッシュを選択
        panel.selected_hashes.insert("12345678".to_string());
        assert!(panel.selected_hashes.contains("12345678"));
        assert_eq!(panel.selected_hashes.len(), 1);

        // 複数選択
        panel.selected_hashes.insert("abcdef12".to_string());
        assert_eq!(panel.selected_hashes.len(), 2);

        // 選択解除
        panel.selected_hashes.remove("12345678");
        assert!(!panel.selected_hashes.contains("12345678"));
        assert_eq!(panel.selected_hashes.len(), 1);
    }

    #[test]
    fn test_decode_panel_status_messages() {
        let mut panel = DecodePanel::default();

        // 成功メッセージ
        panel.status_msg = Some("Success".to_string());
        assert_eq!(panel.status_msg.as_ref().unwrap(), "Success");

        // エラーメッセージ
        panel.error_msg = Some("Error occurred".to_string());
        assert_eq!(panel.error_msg.as_ref().unwrap(), "Error occurred");

        // クリア
        panel.status_msg = None;
        panel.error_msg = None;
        assert!(panel.status_msg.is_none());
        assert!(panel.error_msg.is_none());
    }

    #[test]
    fn test_decode_panel_resolve_output_path() {
        let mut panel = DecodePanel::default();

        // CurrentDir
        panel.output_dir = OutputDir::CurrentDir;
        let path = panel.resolve_output_path("test.txt");
        assert_eq!(path.to_string_lossy(), "test.txt");

        // Custom
        panel.output_dir = OutputDir::Custom("/tmp".to_string());
        let path = panel.resolve_output_path("test.txt");
        assert!(path.to_string_lossy().contains("test.txt"));
    }

    #[test]
    fn test_output_dir_label() {
        assert_eq!(
            OutputDir::SameAsInput.label(),
            "入力ファイルと同じディレクトリ"
        );
        assert_eq!(OutputDir::Downloads.label(), "Downloadsディレクトリ");
        assert_eq!(OutputDir::CurrentDir.label(), "カレントディレクトリ");
        assert_eq!(
            OutputDir::Custom("/path".to_string()).label(),
            "指定ディレクトリ"
        );
    }
}
