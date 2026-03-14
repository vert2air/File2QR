#[cfg(test)]
mod tests {
    use crate::ui::decode_panel::{DecodeMessage, DecodePanel, OutputDir};
    use iced::widget::text_editor;

    // ── デフォルト状態 ────────────────────────────────────────

    #[test]
    fn test_default_state() {
        let panel = DecodePanel::default();
        assert!(panel.input_files.is_empty());
        assert_eq!(panel.file_path_input, "");
        assert!(panel.entries.is_empty());
        assert!(panel.selected_hashes.is_empty());
        assert!(matches!(panel.output_dir, OutputDir::SameAsInput));
        assert_eq!(panel.custom_dir, "");
        assert!(panel.decoded_text.is_none());
        assert!(panel.status_msg.is_none());
        assert!(panel.error_msg.is_none());
        // decoded_content は空（行数 0 or 1）
        assert_eq!(panel.decoded_content.line_count(), 1);
    }

    // ── update() 経由: ファイル操作 ───────────────────────────

    #[test]
    fn test_file_path_input_changed() {
        let mut panel = DecodePanel::default();
        panel.update(DecodeMessage::FilePathInputChanged(
            "test.txt".to_string(),
        ));
        assert_eq!(panel.file_path_input, "test.txt");
    }

    #[test]
    fn test_file_dropped() {
        let mut panel = DecodePanel::default();
        panel.update(DecodeMessage::FileDropped(
            "/nonexistent/file.txt".to_string(),
        ));
        assert_eq!(panel.input_files.len(), 1);
        assert_eq!(panel.input_files[0], "/nonexistent/file.txt");
    }

    #[test]
    fn test_file_dropped_duplicate_ignored() {
        let mut panel = DecodePanel::default();
        panel.update(DecodeMessage::FileDropped("a.txt".to_string()));
        panel.update(DecodeMessage::FileDropped("a.txt".to_string()));
        assert_eq!(panel.input_files.len(), 1);
    }

    #[test]
    fn test_remove_file() {
        let mut panel = DecodePanel::default();
        panel.update(DecodeMessage::FileDropped("a.txt".to_string()));
        panel.update(DecodeMessage::FileDropped("b.txt".to_string()));
        panel.update(DecodeMessage::FileDropped("c.txt".to_string()));

        panel.update(DecodeMessage::RemoveFile(1));
        assert_eq!(panel.input_files.len(), 2);
        assert_eq!(panel.input_files[0], "a.txt");
        assert_eq!(panel.input_files[1], "c.txt");
    }

    #[test]
    fn test_remove_file_out_of_range() {
        let mut panel = DecodePanel::default();
        panel.update(DecodeMessage::FileDropped("a.txt".to_string()));
        panel.update(DecodeMessage::RemoveFile(99));
        assert_eq!(panel.input_files.len(), 1);
    }

    // ── update() 経由: ハッシュ選択 ──────────────────────────

    #[test]
    fn test_hash_toggle_on_off() {
        let mut panel = DecodePanel::default();

        panel.update(DecodeMessage::HashToggled("aabbccdd".to_string(), true));
        assert!(panel.selected_hashes.contains("aabbccdd"));

        panel
            .update(DecodeMessage::HashToggled("aabbccdd".to_string(), false));
        assert!(!panel.selected_hashes.contains("aabbccdd"));
    }

    #[test]
    fn test_hash_toggle_multiple() {
        let mut panel = DecodePanel::default();
        panel.update(DecodeMessage::HashToggled("hash1".to_string(), true));
        panel.update(DecodeMessage::HashToggled("hash2".to_string(), true));
        assert_eq!(panel.selected_hashes.len(), 2);
    }

    // ── update() 経由: 出力先ディレクトリ ────────────────────

    #[test]
    fn test_output_dir_change() {
        let mut panel = DecodePanel::default();

        panel.update(DecodeMessage::OutputDirChanged(OutputDir::Downloads));
        assert!(matches!(panel.output_dir, OutputDir::Downloads));

        panel.update(DecodeMessage::OutputDirChanged(OutputDir::CurrentDir));
        assert!(matches!(panel.output_dir, OutputDir::CurrentDir));

        panel.update(DecodeMessage::OutputDirChanged(OutputDir::SameAsInput));
        assert!(matches!(panel.output_dir, OutputDir::SameAsInput));
    }

    #[test]
    fn test_custom_dir_changed() {
        let mut panel = DecodePanel::default();
        panel.update(DecodeMessage::CustomDirChanged("/my/dir".to_string()));
        assert_eq!(panel.custom_dir, "/my/dir");
        assert!(matches!(panel.output_dir, OutputDir::Custom(_)));
        if let OutputDir::Custom(ref d) = panel.output_dir {
            assert_eq!(d, "/my/dir");
        }
    }

    // ── resolve_output_path ───────────────────────────────────

    #[test]
    fn test_resolve_output_path_current_dir() {
        let mut panel = DecodePanel::default();
        panel.output_dir = OutputDir::CurrentDir;
        let path = panel.resolve_output_path("out.txt");
        assert_eq!(path.to_string_lossy(), "out.txt");
    }

    #[test]
    fn test_resolve_output_path_custom() {
        let mut panel = DecodePanel::default();
        panel.output_dir = OutputDir::Custom("/tmp/out".to_string());
        let path = panel.resolve_output_path("result.bin");
        assert!(path.ends_with("result.bin"));
        assert!(path.to_string_lossy().contains("tmp"));
    }

    #[test]
    fn test_resolve_output_path_same_as_input_no_files() {
        let mut panel = DecodePanel::default();
        panel.output_dir = OutputDir::SameAsInput;
        let path = panel.resolve_output_path("fallback.txt");
        assert_eq!(path.to_string_lossy(), "fallback.txt");
    }

    // ── TextEditorAction: 読み取り専用 ────────────────────────

    #[test]
    fn test_text_editor_readonly_blocks_edit() {
        let mut panel = DecodePanel::default();
        panel.decoded_content = text_editor::Content::with_text("Hello");

        let before_lines = panel.decoded_content.line_count();

        // Edit アクションは無視される → 行数変化なし
        panel.update(DecodeMessage::TextEditorAction(
            text_editor::Action::Edit(text_editor::Edit::Insert('X')),
        ));
        assert_eq!(panel.decoded_content.line_count(), before_lines);
    }

    #[test]
    fn test_text_editor_readonly_blocks_delete() {
        let mut panel = DecodePanel::default();
        panel.decoded_content = text_editor::Content::with_text("Hello");

        let before_lines = panel.decoded_content.line_count();

        panel.update(DecodeMessage::TextEditorAction(
            text_editor::Action::Edit(text_editor::Edit::Backspace),
        ));
        assert_eq!(panel.decoded_content.line_count(), before_lines);
    }

    #[test]
    fn test_text_editor_allows_move() {
        let mut panel = DecodePanel::default();
        panel.decoded_content = text_editor::Content::with_text("Hello World");

        // Move アクションはパニックせず通る
        panel.update(DecodeMessage::TextEditorAction(
            text_editor::Action::Move(text_editor::Motion::Right),
        ));
    }

    #[test]
    fn test_text_editor_allows_select_all() {
        let mut panel = DecodePanel::default();
        panel.decoded_content = text_editor::Content::with_text("Hello World");

        // SelectAll はパニックせず通る
        panel.update(DecodeMessage::TextEditorAction(
            text_editor::Action::SelectAll,
        ));
    }

    // ── OutputDir::label ──────────────────────────────────────

    #[test]
    fn test_output_dir_label() {
        assert_eq!(
            OutputDir::SameAsInput.label(),
            "入力ファイルと同じディレクトリ"
        );
        assert_eq!(OutputDir::Downloads.label(), "Downloadsディレクトリ");
        assert_eq!(OutputDir::CurrentDir.label(), "カレントディレクトリ");
        assert_eq!(
            OutputDir::Custom("/x".to_string()).label(),
            "指定ディレクトリ"
        );
    }

    // ── ステータス/エラーメッセージ ──────────────────────────

    #[test]
    fn test_status_and_error_messages() {
        let mut panel = DecodePanel::default();

        panel.status_msg = Some("成功".to_string());
        panel.error_msg = Some("失敗".to_string());
        assert_eq!(panel.status_msg.as_deref(), Some("成功"));
        assert_eq!(panel.error_msg.as_deref(), Some("失敗"));

        panel.status_msg = None;
        panel.error_msg = None;
        assert!(panel.status_msg.is_none());
        assert!(panel.error_msg.is_none());
    }
}
