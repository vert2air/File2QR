/// ファイル内容からフラグメント行を抽出するユーティリティ

pub fn extract_lines(content: &str) -> Vec<&str> {
    content.lines().collect()
}
