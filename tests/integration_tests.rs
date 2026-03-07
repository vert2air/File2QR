/// 統合テスト: encode と decode の完全なラウンドトリップ
use file2qr::{decode, encode};

#[test]
fn integration_test_roundtrip_realistic_file() {
    // 実際のファイルを想定したテスト
    let test_data = include_bytes!("../Cargo.toml");
    let filename = "Cargo.toml";

    // エンコード
    let input = encode::EncodeInput {
        data: test_data.to_vec(),
        filename: filename.to_string(),
        compress: false,
        ec_level: encode::EcLevel::M,
    };

    let encode_result = encode::encode(input).expect("encode should succeed");

    // QRコードが生成されている
    assert!(
        !encode_result.fragments.is_empty(),
        "should generate at least one fragment"
    );

    // デコード
    let lines: Vec<&str> = encode_result
        .fragments
        .iter()
        .map(|s| s.as_str())
        .collect();
    let entries = decode::parse_lines(&lines);

    assert_eq!(entries.len(), 1, "should have exactly one hash entry");

    let entry = entries.values().next().unwrap();
    assert!(entry.is_complete(), "all fragments should be present");

    // データ復元
    let decoded_data = decode::reconstruct(entry).expect("reconstruct should succeed");

    // 元データと一致
    assert_eq!(
        decoded_data, test_data,
        "decoded data should match original Cargo.toml"
    );
}

#[test]
fn integration_test_multiple_ec_levels() {
    let test_data = b"Integration test data for all error correction levels";

    for ec_level in encode::EcLevel::all() {
        let input = encode::EncodeInput {
            data: test_data.to_vec(),
            filename: format!("test_{:?}.txt", ec_level),
            compress: false,
            ec_level: *ec_level,
        };

        let encode_result = encode::encode(input).unwrap();
        let lines: Vec<&str> = encode_result
            .fragments
            .iter()
            .map(|s| s.as_str())
            .collect();

        let entries = decode::parse_lines(&lines);
        let entry = entries.values().next().unwrap();
        let decoded_data = decode::reconstruct(entry).unwrap();

        assert_eq!(
            decoded_data, test_data,
            "roundtrip failed for {:?}",
            ec_level
        );
    }
}

#[test]
fn integration_test_large_file() {
    // 大きなファイル（100KB）
    let test_data = vec![0xAB; 100_000];
    let filename = "large_file.bin";

    let input = encode::EncodeInput {
        data: test_data.clone(),
        filename: filename.to_string(),
        compress: true, // 圧縮も有効化
        ec_level: encode::EcLevel::L,
    };

    let encode_result = encode::encode(input).expect("encode large file");

    // 複数のQRコードに分割されているはず
    assert!(
        encode_result.fragments.len() > 10,
        "large file should be split into many fragments"
    );

    let lines: Vec<&str> = encode_result
        .fragments
        .iter()
        .map(|s| s.as_str())
        .collect();
    let entries = decode::parse_lines(&lines);
    let entry = entries.values().next().unwrap();

    assert!(entry.is_complete());
    assert_eq!(entry.compressed, Some(true));

    let decoded_data = decode::reconstruct(entry).expect("reconstruct large file");
    assert_eq!(decoded_data, test_data);
}

#[test]
fn integration_test_compression_effectiveness() {
    // 圧縮効果があるデータ
    let repetitive_data = b"AAAAAAAAAA".repeat(1000);

    // 圧縮なし
    let input_no_compress = encode::EncodeInput {
        data: repetitive_data.clone(),
        filename: "no_compress.txt".to_string(),
        compress: false,
        ec_level: encode::EcLevel::M,
    };
    let result_no_compress = encode::encode(input_no_compress).unwrap();

    // 圧縮あり
    let input_compress = encode::EncodeInput {
        data: repetitive_data.clone(),
        filename: "compress.txt".to_string(),
        compress: true,
        ec_level: encode::EcLevel::M,
    };
    let result_compress = encode::encode(input_compress).unwrap();

    // 圧縮した方がフラグメント数が少ないはず
    assert!(
        result_compress.fragments.len() < result_no_compress.fragments.len(),
        "compression should reduce fragment count"
    );

    // 両方とも正しく復元できる
    for (name, result) in [
        ("no_compress", result_no_compress),
        ("compress", result_compress),
    ] {
        let lines: Vec<&str> = result.fragments.iter().map(|s| s.as_str()).collect();
        let entries = decode::parse_lines(&lines);
        let entry = entries.values().next().unwrap();
        let decoded = decode::reconstruct(entry).unwrap();
        assert_eq!(decoded, repetitive_data, "{} failed", name);
    }
}

#[test]
fn integration_test_fragment_shuffle() {
    // フラグメントをシャッフルしても復元できる
    let test_data = b"Fragment order should not matter";
    let input = encode::EncodeInput {
        data: test_data.to_vec(),
        filename: "shuffle_test.txt".to_string(),
        compress: false,
        ec_level: encode::EcLevel::H,
    };

    let encode_result = encode::encode(input).unwrap();

    // フラグメントを逆順にする
    let mut reversed_fragments = encode_result.fragments.clone();
    reversed_fragments.reverse();

    let lines: Vec<&str> = reversed_fragments.iter().map(|s| s.as_str()).collect();
    let entries = decode::parse_lines(&lines);
    let entry = entries.values().next().unwrap();

    let decoded_data = decode::reconstruct(entry).unwrap();
    assert_eq!(decoded_data, test_data, "shuffled fragments should work");
}
