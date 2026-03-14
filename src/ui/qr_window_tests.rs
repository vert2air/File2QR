#[cfg(test)]
mod tests {
    use crate::encode::EcLevel;
    use crate::ui::qr_window::{QrWindow, QrWindowMessage};

    // ── 初期化 ────────────────────────────────────────────────

    #[test]
    fn test_initialization() {
        let win = QrWindow::new_for_test(
            vec!["a".to_string(), "b".to_string()],
            EcLevel::M,
        );
        assert_eq!(win.fragments.len(), 2);
        assert_eq!(win.ec_level, EcLevel::M);
        assert_eq!(win.rows, 2);
        assert_eq!(win.cols, 3);
        assert_eq!(win.page, 0);
        assert_eq!(win.scale, 2); // new_for_test のデフォルト
        assert!(win.open);
        assert!(!win.fullscreen);
    }

    #[test]
    fn test_empty_fragments() {
        let win = QrWindow::new_for_test(vec![], EcLevel::L);
        assert_eq!(win.fragments.len(), 0);
        assert_eq!(win.total_pages(), 0);
        assert_eq!(win.page_range().count(), 0);
    }

    #[test]
    fn test_single_fragment() {
        let win = QrWindow::new_for_test(vec!["x".to_string()], EcLevel::H);
        assert_eq!(win.total_pages(), 1);
        assert_eq!(win.page_range().collect::<Vec<_>>(), vec![0]);
    }

    // ── ページ計算 ────────────────────────────────────────────

    #[test]
    fn test_total_pages() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string(); 20], EcLevel::L);
        win.rows = 2;
        win.cols = 3; // 6個/ページ → ceil(20/6) = 4
        assert_eq!(win.total_pages(), 4);
    }

    #[test]
    fn test_total_pages_exact_multiple() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string(); 6], EcLevel::L);
        win.rows = 2;
        win.cols = 3;
        assert_eq!(win.total_pages(), 1);
    }

    #[test]
    fn test_page_range_first() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string(); 10], EcLevel::L);
        win.rows = 2;
        win.cols = 2; // 4個/ページ
        win.page = 0;
        assert_eq!(win.page_range().collect::<Vec<_>>(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_page_range_middle() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string(); 10], EcLevel::L);
        win.rows = 2;
        win.cols = 2;
        win.page = 1;
        assert_eq!(win.page_range().collect::<Vec<_>>(), vec![4, 5, 6, 7]);
    }

    #[test]
    fn test_page_range_last_partial() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string(); 10], EcLevel::L);
        win.rows = 2;
        win.cols = 2;
        win.page = 2; // 最後のページ: 2個だけ
        assert_eq!(win.page_range().collect::<Vec<_>>(), vec![8, 9]);
    }

    // ── update() 経由のページ移動 ─────────────────────────────

    #[test]
    fn test_next_page() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string(); 10], EcLevel::L);
        win.rows = 2;
        win.cols = 2; // 3ページ
        assert_eq!(win.page, 0);

        win.update(QrWindowMessage::NextPage);
        assert_eq!(win.page, 1);

        win.update(QrWindowMessage::NextPage);
        assert_eq!(win.page, 2);

        // 最終ページでは進まない
        win.update(QrWindowMessage::NextPage);
        assert_eq!(win.page, 2);
    }

    #[test]
    fn test_prev_page() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string(); 10], EcLevel::L);
        win.rows = 2;
        win.cols = 2;
        win.page = 2;

        win.update(QrWindowMessage::PrevPage);
        assert_eq!(win.page, 1);

        win.update(QrWindowMessage::PrevPage);
        assert_eq!(win.page, 0);

        // 先頭ページでは戻らない
        win.update(QrWindowMessage::PrevPage);
        assert_eq!(win.page, 0);
    }

    // ── update() 経由の行・列変更 ─────────────────────────────

    #[test]
    fn test_rows_inc_dec() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string()], EcLevel::L);
        assert_eq!(win.rows, 2);

        win.update(QrWindowMessage::RowsInc);
        assert_eq!(win.rows, 3);

        win.update(QrWindowMessage::RowsDec);
        assert_eq!(win.rows, 2);

        // 1未満にはならない
        win.rows = 1;
        win.update(QrWindowMessage::RowsDec);
        assert_eq!(win.rows, 1);
    }

    #[test]
    fn test_cols_inc_dec() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string()], EcLevel::L);
        assert_eq!(win.cols, 3);

        win.update(QrWindowMessage::ColsInc);
        assert_eq!(win.cols, 4);

        win.update(QrWindowMessage::ColsDec);
        assert_eq!(win.cols, 3);

        // 1未満にはならない
        win.cols = 1;
        win.update(QrWindowMessage::ColsDec);
        assert_eq!(win.cols, 1);
    }

    // ── update() 経由のスケール変更 ───────────────────────────

    #[test]
    fn test_scale_inc_dec() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string()], EcLevel::L);
        assert_eq!(win.scale, 2);

        win.update(QrWindowMessage::ScaleInc);
        assert_eq!(win.scale, 3);

        win.update(QrWindowMessage::ScaleDec);
        assert_eq!(win.scale, 2);

        // 1未満にはならない
        win.scale = 1;
        win.update(QrWindowMessage::ScaleDec);
        assert_eq!(win.scale, 1);
    }

    #[test]
    fn test_scale_max() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string()], EcLevel::L);
        win.scale = 8;
        win.update(QrWindowMessage::ScaleInc);
        assert_eq!(win.scale, 8); // 上限8を超えない
    }

    // ── update() 経由のClose ──────────────────────────────────

    #[test]
    fn test_close() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string()], EcLevel::L);
        assert!(win.open);
        win.update(QrWindowMessage::Close);
        assert!(!win.open);
    }

    // ── per_page ──────────────────────────────────────────────

    #[test]
    fn test_per_page() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string()], EcLevel::L);
        win.rows = 3;
        win.cols = 4;
        assert_eq!(win.per_page(), 12);
    }

    // ── fullscreen フィールド（互換保持） ─────────────────────

    #[test]
    fn test_fullscreen_field_exists() {
        let mut win =
            QrWindow::new_for_test(vec!["x".to_string()], EcLevel::L);
        assert!(!win.fullscreen);
        win.fullscreen = true;
        assert!(win.fullscreen);
    }
}
