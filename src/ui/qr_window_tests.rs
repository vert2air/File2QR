#[cfg(test)]
mod tests {
    use super::QrWindow;
    use crate::encode::EcLevel;

    #[test]
    fn test_qr_window_initialization() {
        let fragments = vec!["fragment1".to_string(), "fragment2".to_string()];
        
        let window = QrWindow::new(fragments.clone(), EcLevel::M);
        
        assert_eq!(window.fragments.len(), 2);
        assert_eq!(window.ec_level, EcLevel::M);
        assert_eq!(window.rows, 2);
        assert_eq!(window.cols, 3);
        assert_eq!(window.page, 0);
        assert!(window.open);
        assert!(!window.fullscreen);
    }

    #[test]
    fn test_qr_window_total_pages() {
        let fragments = vec!["1".to_string(); 20]; // 20個のフラグメント
        let mut window = QrWindow::new(fragments, EcLevel::M);
        
        // 2行3列 = 6個/ページ
        window.rows = 2;
        window.cols = 3;
        
        let total = window.total_pages();
        assert_eq!(total, 4); // ceil(20 / 6) = 4
    }

    #[test]
    fn test_qr_window_page_range() {
        let fragments = vec!["1".to_string(); 10];
        let mut window = QrWindow::new(fragments, EcLevel::M);
        
        window.rows = 2;
        window.cols = 2; // 4個/ページ
        window.page = 0;
        
        let range: Vec<usize> = window.page_range().collect();
        assert_eq!(range, vec![0, 1, 2, 3]);
        
        // 2ページ目
        window.page = 1;
        let range: Vec<usize> = window.page_range().collect();
        assert_eq!(range, vec![4, 5, 6, 7]);
        
        // 最後のページ（2個だけ）
        window.page = 2;
        let range: Vec<usize> = window.page_range().collect();
        assert_eq!(range, vec![8, 9]);
    }

    #[test]
    fn test_qr_window_next_page() {
        let fragments = vec!["1".to_string(); 10];
        let mut window = QrWindow::new(fragments, EcLevel::M);
        
        window.rows = 2;
        window.cols = 2;
        
        assert_eq!(window.page, 0);
        assert_eq!(window.total_pages(), 3);
        
        // ページ移動はUI操作が必要なのでロジックのみテスト
        let old_page = window.page;
        if window.page + 1 < window.total_pages() {
            window.page += 1;
        }
        assert_eq!(window.page, old_page + 1);
    }

    #[test]
    fn test_qr_window_prev_page() {
        let fragments = vec!["1".to_string(); 10];
        let mut window = QrWindow::new(fragments, EcLevel::M);
        
        window.page = 2;
        
        let old_page = window.page;
        if window.page > 0 {
            window.page -= 1;
        }
        assert_eq!(window.page, old_page - 1);
        
        // 0ページより前には行けない
        window.page = 0;
        if window.page > 0 {
            window.page -= 1;
        }
        assert_eq!(window.page, 0);
    }

    #[test]
    fn test_qr_window_rows_cols_adjustment() {
        let fragments = vec!["1".to_string(); 100];
        let mut window = QrWindow::new(fragments, EcLevel::M);
        
        // 行数増加
        let old_rows = window.rows;
        window.rows += 1;
        assert_eq!(window.rows, old_rows + 1);
        
        // 列数増加
        let old_cols = window.cols;
        window.cols += 1;
        assert_eq!(window.cols, old_cols + 1);
        
        // 最小値チェック（1以下にならない）
        window.rows = 1;
        if window.rows > 1 {
            window.rows -= 1;
        }
        assert_eq!(window.rows, 1);
    }

    #[test]
    fn test_qr_window_fullscreen_toggle() {
        let fragments = vec!["1".to_string()];
        let mut window = QrWindow::new(fragments, EcLevel::M);
        
        assert!(!window.fullscreen);
        
        // トグル
        window.fullscreen = true;
        assert!(window.fullscreen);
        
        window.fullscreen = false;
        assert!(!window.fullscreen);
    }

    #[test]
    fn test_qr_window_scale_env_var() {
        // 環境変数のテスト（実際の値は実行環境依存）
        std::env::remove_var("FILE2QR_SCALE");
        
        let fragments = vec!["1".to_string()];
        let window = QrWindow::new(fragments, EcLevel::M);
        
        // デフォルトは2
        assert_eq!(window.scale, 2);
        
        // 環境変数設定
        std::env::set_var("FILE2QR_SCALE", "5");
        let fragments2 = vec!["1".to_string()];
        let window2 = QrWindow::new(fragments2, EcLevel::M);
        assert_eq!(window2.scale, 5);
        
        // クリーンアップ
        std::env::remove_var("FILE2QR_SCALE");
    }

    #[test]
    fn test_qr_window_empty_fragments() {
        let fragments = vec![];
        let window = QrWindow::new(fragments, EcLevel::L);
        
        assert_eq!(window.fragments.len(), 0);
        assert_eq!(window.total_pages(), 0);
    }

    #[test]
    fn test_qr_window_single_fragment() {
        let fragments = vec!["single".to_string()];
        let window = QrWindow::new(fragments, EcLevel::H);
        
        assert_eq!(window.fragments.len(), 1);
        assert_eq!(window.total_pages(), 1);
        
        let range: Vec<usize> = window.page_range().collect();
        assert_eq!(range, vec![0]);
    }
}
