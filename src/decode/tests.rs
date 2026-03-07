#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fragment_line_valid() {
        let line = "abcd1234:001:003:example.txt::base64data";
        let frag = parse_lines(&[line]);

        assert_eq!(frag.len(), 1);
        let entry = frag.values().next().unwrap();
        assert_eq!(entry.hash, "abcd1234");
        assert_eq!(entry.fragments.len(), 1);
        assert!(entry.fragments.contains_key(&1));
    }

    #[test]
    fn test_parse_fragment_line_multiple() {
        let lines = vec![
            "12345678:001:003:test.txt::chunk1",
            "12345678:002:chunk2",
            "12345678:003:chunk3",
        ];

        let entries = parse_lines(&lines);
        assert_eq!(entries.len(), 1);

        let entry = entries.get("12345678").unwrap();
        assert_eq!(entry.qr_num, Some(3));
        assert_eq!(entry.fragments.len(), 3);
        assert_eq!(entry.filename.as_ref().unwrap(), "test.txt");
    }

    #[test]
    fn test_parse_fragment_line_invalid() {
        let invalid_lines = vec![
            "invalid",
            "tooshort:001",
            "notahash:001:data",
            "12345:001:data", // hash too short
        ];

        for line in invalid_lines {
            let entries = parse_lines(&[line]);
            assert_eq!(
                entries.len(),
                0,
                "invalid line should not create entries: {}",
                line
            );
        }
    }

    #[test]
    fn test_missing_indices() {
        let lines = vec![
            "abcdef12:001:005:file.txt::data1",
            "abcdef12:002:data2",
            "abcdef12:004:data4",
            // フラグメント3と5が欠落
        ];

        let entries = parse_lines(&lines);
        let entry = entries.get("abcdef12").unwrap();

        let missing = entry.missing_indices();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&3));
        assert!(missing.contains(&5));
    }

    #[test]
    fn test_is_complete() {
        // 完全なケース
        let complete_lines = vec![
            "aabbccdd:001:003:complete.txt::data1",
            "aabbccdd:002:data2",
            "aabbccdd:003:data3",
        ];

        let entries = parse_lines(&complete_lines);
        let entry = entries.get("aabbccdd").unwrap();
        assert!(entry.is_complete());

        // 不完全なケース
        let incomplete_lines = vec![
            "11223344:001:003:incomplete.txt::data1",
            "11223344:003:data3",
            // フラグメント2が欠落
        ];

        let entries = parse_lines(&incomplete_lines);
        let entry = entries.get("11223344").unwrap();
        assert!(!entry.is_complete());
    }

    #[test]
    fn test_extract_meta() {
        let chunk = "003:example.txt::YWJj";
        if let Some((qr_num, filename, compressed)) = extract_meta(chunk) {
            assert_eq!(qr_num, 3);
            assert_eq!(filename, "example.txt");
            assert_eq!(compressed, false);
        } else {
            panic!("extract_meta should succeed");
        }

        // 圧縮フラグありのケース
        let chunk_compressed = "005:test.bin:xv:YWJj";
        if let Some((qr_num, filename, compressed)) = extract_meta(chunk_compressed) {
            assert_eq!(qr_num, 5);
            assert_eq!(filename, "test.bin");
            assert_eq!(compressed, true);
        } else {
            panic!("extract_meta should succeed for compressed");
        }
    }

    #[test]
    fn test_extract_meta_with_special_chars() {
        // パーセントエンコードされたファイル名
        let chunk = "001:test%20file.txt::data";
        if let Some((_, filename, _)) = extract_meta(chunk) {
            assert_eq!(filename, "test file.txt");
        } else {
            panic!("extract_meta should decode percent-encoded filename");
        }
    }

    #[test]
    fn test_multiple_hashes() {
        // 異なるハッシュ値のフラグメントが混在
        let lines = vec![
            "11111111:001:002:file1.txt::data1a",
            "22222222:001:002:file2.txt::data2a",
            "11111111:002:data1b",
            "22222222:002:data2b",
        ];

        let entries = parse_lines(&lines);
        assert_eq!(entries.len(), 2);

        assert!(entries.contains_key("11111111"));
        assert!(entries.contains_key("22222222"));

        let entry1 = entries.get("11111111").unwrap();
        assert_eq!(entry1.filename.as_ref().unwrap(), "file1.txt");
        assert!(entry1.is_complete());

        let entry2 = entries.get("22222222").unwrap();
        assert_eq!(entry2.filename.as_ref().unwrap(), "file2.txt");
        assert!(entry2.is_complete());
    }

    #[test]
    fn test_fragment_order_independence() {
        // フラグメントが順不同でも正しく解析できる
        let lines = vec![
            "aaaabbbb:003:data3",
            "aaaabbbb:001:003:file.txt::data1",
            "aaaabbbb:002:data2",
        ];

        let entries = parse_lines(&lines);
        let entry = entries.get("aaaabbbb").unwrap();

        assert!(entry.is_complete());
        assert_eq!(entry.qr_num, Some(3));
        assert_eq!(entry.fragments.get(&1).unwrap(), "003:file.txt::data1");
        assert_eq!(entry.fragments.get(&2).unwrap(), "data2");
        assert_eq!(entry.fragments.get(&3).unwrap(), "data3");
    }

    #[test]
    fn test_duplicate_fragments() {
        // 同じフラグメント番号が重複している場合（後勝ち）
        let lines = vec![
            "12345678:001:002:file.txt::olddata",
            "12345678:001:002:file.txt::newdata",
            "12345678:002:data2",
        ];

        let entries = parse_lines(&lines);
        let entry = entries.get("12345678").unwrap();

        // 最後に追加されたものが保持される
        assert_eq!(
            entry.fragments.get(&1).unwrap(),
            "002:file.txt::newdata"
        );
    }

    #[test]
    fn test_parse_lines_with_noise() {
        // ノイズ（無効な行）が混在していても有効な行だけ解析できる
        let lines = vec![
            "some random text",
            "abcd1234:001:002:valid.txt::data1",
            "",
            "invalid:line:here",
            "abcd1234:002:data2",
            "# comment line",
        ];

        let entries = parse_lines(&lines);
        assert_eq!(entries.len(), 1);

        let entry = entries.get("abcd1234").unwrap();
        assert_eq!(entry.fragments.len(), 2);
        assert!(entry.is_complete());
    }
}
