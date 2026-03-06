pub mod fragment;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use sha2::{Digest, Sha256};

/// エラー訂正レベル
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EcLevel {
    L,
    M,
    Q,
    H,
}

impl EcLevel {
    /// レベルの記号文字列
    pub fn label(&self) -> &'static str {
        match self {
            EcLevel::L => "L",
            EcLevel::M => "M",
            EcLevel::Q => "Q",
            EcLevel::H => "H",
        }
    }

    /// 仕様書記載の最大データ容量 (byte) - 実測に基づく安全値
    pub fn max_bytes(&self) -> usize {
        // qrcodeクレートの実際の制限は理論値より若干小さいため、
        // 安全マージンを確保（約5%減）
        match self {
            EcLevel::L => 2800, // 理論値 2953
            EcLevel::M => 2200, // 理論値 2331
            EcLevel::Q => 1580, // 理論値 1663
            EcLevel::H => 1210, // 理論値 1272
        }
    }

    /// QRコード1個当たりのエンコード可能なデータ長 (qr_cap = max_bytes - 13)
    pub fn qr_cap(&self) -> usize {
        self.max_bytes().saturating_sub(13)
    }

    pub fn all() -> &'static [EcLevel] {
        &[EcLevel::L, EcLevel::M, EcLevel::Q, EcLevel::H]
    }
}

/// エンコード入力
pub struct EncodeInput {
    pub data: Vec<u8>,
    pub filename: String,
    pub compress: bool,
    pub ec_level: EcLevel,
}

/// エンコード結果
pub struct EncodeResult {
    pub fragments: Vec<String>,
}

/// メインエンコード処理
pub fn encode(input: EncodeInput) -> Result<EncodeResult, String> {
    // 1. 圧縮
    let data = if input.compress {
        compress_xz(&input.data)?
    } else {
        input.data.clone()
    };

    // 2. Base64変換
    let b64 = STANDARD.encode(&data);

    // 3. ファイル名をパーセントエンコード
    let encoded_filename =
        utf8_percent_encode(&input.filename, NON_ALPHANUMERIC).to_string();

    // 4. 圧縮フラグ
    let compress_flag = if input.compress { "xv" } else { "" };

    // 5. hash_data 作成
    let hash_data = format!("{}:{}:{}", encoded_filename, compress_flag, b64);

    // 6. SHA-256 先頭4byte → 8桁16進
    let hash_str = {
        let mut hasher = Sha256::new();
        hasher.update(hash_data.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..4])
    };

    // 7. QRコード数の計算
    let qr_cap = input.ec_level.qr_cap();

    // 実際に必要なQRコード数を計算
    // 各フラグメントは "hash:NNN:chunk" の形式
    // hash(8) + :(1) + NNN(3) + :(1) = 13文字は固定
    // chunk部分の最大長 = qr_cap
    let qr_num = hash_data.len().div_ceil(qr_cap);

    // 8. qr_data 作成（実際のqr_numで）
    let qr_data_full = format!("{:0>3}:{}", qr_num, &hash_data);

    // 9. フラグメント生成（1オリジン）
    let fragments: Vec<String> = (0..qr_num)
        .map(|i| {
            let start = i * qr_cap;
            let end = ((i + 1) * qr_cap).min(qr_data_full.len());
            let chunk = &qr_data_full[start..end];
            // フラグメント番号は1オリジン（i+1）
            let fragment = format!("{}:{:0>3}:{}", hash_str, i + 1, chunk);

            // デバッグ: 長さを確認
            if fragment.len() > input.ec_level.max_bytes() {
                eprintln!(
                    "WARNING: Fragment {} 長さオーバー: {} > {} (max_bytes)",
                    i + 1,
                    fragment.len(),
                    input.ec_level.max_bytes()
                );
                eprintln!(
                    "  hash={}, index={}, chunk_len={}",
                    hash_str,
                    i + 1,
                    chunk.len()
                );
            }

            fragment
        })
        .collect();

    Ok(EncodeResult { fragments })
}

fn compress_xz(data: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Write;
    use xz2::write::XzEncoder;

    let mut encoder = XzEncoder::new(Vec::new(), 6);
    encoder.write_all(data).map_err(|e| format!("xz圧縮エラー: {}", e))?;
    encoder.finish().map_err(|e| format!("xz圧縮完了エラー: {}", e))
}
