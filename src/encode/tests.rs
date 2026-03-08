#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::decode;

    #[test]
    fn test_roundtrip_small_text() {
        // 小さなテキストデータ
        let original_data = b"Hello, World!";
        let filename = "test.txt";

        // エンコード
        let input = EncodeInput {
            data: original_data.to_vec(),
            filename: filename.to_string(),
            compress: false,
            ec_level: EcLevel::M,
        };

        let result = encode(input).expect("encode failed");
        assert!(!result.fragments.is_empty(), "fragments should not be empty");

        // フラグメントをテキスト行として扱う
        let lines: Vec<&str> =
            result.fragments.iter().map(|s| s.as_str()).collect();

        // デコード
        let entries = decode::parse_lines(&lines);
        assert_eq!(entries.len(), 1, "should have exactly one hash entry");

        let entry = entries.values().next().unwrap();
        assert!(entry.is_complete(), "entry should be complete");
        assert_eq!(
            entry.filename.as_ref().unwrap(),
            filename,
            "filename mismatch"
        );

        // データ復元
        let decoded_data =
            decode::reconstruct(entry).expect("reconstruct failed");
        assert_eq!(
            decoded_data, original_data,
            "decoded data should match original"
        );
    }

    #[test]
    fn test_roundtrip_with_compression() {
        // 圧縮あり
        let original_data = b"A".repeat(1000); // 圧縮効果がある繰り返しデータ
        let filename = "compressed.bin";

        let input = EncodeInput {
            data: original_data.to_vec(),
            filename: filename.to_string(),
            compress: true,
            ec_level: EcLevel::L,
        };

        let result = encode(input).expect("encode failed");
        let lines: Vec<&str> =
            result.fragments.iter().map(|s| s.as_str()).collect();

        let entries = decode::parse_lines(&lines);
        let entry = entries.values().next().unwrap();

        assert_eq!(
            entry.compressed,
            Some(true),
            "should be marked as compressed"
        );

        let decoded_data =
            decode::reconstruct(entry).expect("reconstruct failed");
        assert_eq!(decoded_data, original_data, "decompressed data mismatch");
    }

    #[test]
    fn test_roundtrip_large_data() {
        // 複数QRコードに分割されるサイズ
        let original_data = b"X".repeat(10000);
        let filename = "large.dat";

        let input = EncodeInput {
            data: original_data.to_vec(),
            filename: filename.to_string(),
            compress: false,
            ec_level: EcLevel::M,
        };

        let result = encode(input).expect("encode failed");
        assert!(
            result.fragments.len() > 1,
            "large data should create multiple fragments"
        );

        let lines: Vec<&str> =
            result.fragments.iter().map(|s| s.as_str()).collect();
        let entries = decode::parse_lines(&lines);
        let entry = entries.values().next().unwrap();

        assert!(entry.is_complete(), "all fragments should be present");

        let decoded_data =
            decode::reconstruct(entry).expect("reconstruct failed");
        assert_eq!(decoded_data, original_data, "large data mismatch");
    }

    #[test]
    fn test_roundtrip_all_ec_levels() {
        // 全エラー訂正レベルでテスト
        let original_data = b"Test data for all EC levels";
        let filename = "ec_test.txt";

        for ec_level in EcLevel::all() {
            let input = EncodeInput {
                data: original_data.to_vec(),
                filename: filename.to_string(),
                compress: false,
                ec_level: *ec_level,
            };

            let result = encode(input)
                .expect(&format!("encode failed for {:?}", ec_level));
            let lines: Vec<&str> =
                result.fragments.iter().map(|s| s.as_str()).collect();

            let entries = decode::parse_lines(&lines);
            let entry = entries.values().next().unwrap();

            let decoded_data = decode::reconstruct(entry)
                .expect(&format!("reconstruct failed for {:?}", ec_level));
            assert_eq!(
                decoded_data, original_data,
                "data mismatch for {:?}",
                ec_level
            );
        }
    }

    #[test]
    fn test_roundtrip_special_filename() {
        // 特殊文字を含むファイル名
        let original_data = b"data";
        let filename = "test file (1) [copy].txt"; // スペース、括弧を含む

        let input = EncodeInput {
            data: original_data.to_vec(),
            filename: filename.to_string(),
            compress: false,
            ec_level: EcLevel::M,
        };

        let result = encode(input).expect("encode failed");
        let lines: Vec<&str> =
            result.fragments.iter().map(|s| s.as_str()).collect();

        let entries = decode::parse_lines(&lines);
        let entry = entries.values().next().unwrap();

        assert_eq!(
            entry.filename.as_ref().unwrap(),
            filename,
            "special filename should be preserved"
        );

        let decoded_data =
            decode::reconstruct(entry).expect("reconstruct failed");
        assert_eq!(decoded_data, original_data);
    }

    #[test]
    fn test_roundtrip_binary_data() {
        // バイナリデータ（全バイト値）
        let original_data: Vec<u8> = (0..=255).collect();
        let filename = "binary.bin";

        let input = EncodeInput {
            data: original_data.clone(),
            filename: filename.to_string(),
            compress: false,
            ec_level: EcLevel::H,
        };

        let result = encode(input).expect("encode failed");
        let lines: Vec<&str> =
            result.fragments.iter().map(|s| s.as_str()).collect();

        let entries = decode::parse_lines(&lines);
        let entry = entries.values().next().unwrap();

        let decoded_data =
            decode::reconstruct(entry).expect("reconstruct failed");
        assert_eq!(decoded_data, original_data, "binary data mismatch");
    }

    #[test]
    fn test_missing_fragments() {
        // フラグメント欠損のテスト
        let original_data = b"X".repeat(10000);
        let filename = "incomplete.dat";

        let input = EncodeInput {
            data: original_data.to_vec(),
            filename: filename.to_string(),
            compress: false,
            ec_level: EcLevel::M,
        };

        let result = encode(input).expect("encode failed");
        assert!(
            result.fragments.len() > 2,
            "need multiple fragments for this test"
        );

        // 最後のフラグメントを削除
        let incomplete_lines: Vec<&str> = result.fragments
            [..result.fragments.len() - 1]
            .iter()
            .map(|s| s.as_str())
            .collect();

        let entries = decode::parse_lines(&incomplete_lines);
        let entry = entries.values().next().unwrap();

        assert!(!entry.is_complete(), "should not be complete");
        assert!(
            !entry.missing_indices().is_empty(),
            "should have missing indices"
        );

        // 復元は失敗するはず
        assert!(
            decode::reconstruct(entry).is_err(),
            "reconstruct should fail with missing fragments"
        );
    }

    #[test]
    fn test_empty_data() {
        // 空データ
        let original_data = b"";
        let filename = "empty.txt";

        let input = EncodeInput {
            data: original_data.to_vec(),
            filename: filename.to_string(),
            compress: false,
            ec_level: EcLevel::L,
        };

        let result = encode(input).expect("encode failed");
        let lines: Vec<&str> =
            result.fragments.iter().map(|s| s.as_str()).collect();

        let entries = decode::parse_lines(&lines);
        let entry = entries.values().next().unwrap();

        let decoded_data =
            decode::reconstruct(entry).expect("reconstruct failed");
        assert_eq!(
            decoded_data, original_data,
            "empty data should round-trip"
        );
    }
}
