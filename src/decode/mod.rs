pub mod fragment;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use percent_encoding::percent_decode_str;
use std::collections::HashMap;

/// 1つのフラグメント行を解析した結果
#[derive(Debug, Clone)]
pub struct ParsedFragment {
    pub hash: String,
    pub index: usize,
    pub chunk: String,
}

/// hash値ごとに集約したデータ
#[derive(Debug, Clone)]
pub struct HashEntry {
    pub hash: String,
    pub qr_num: Option<usize>,       // 0番目のフラグメントから判明
    pub filename: Option<String>,    // 0番目のフラグメントから判明
    pub compressed: Option<bool>,    // 0番目のフラグメントから判明
    pub fragments: HashMap<usize, String>, // index -> chunk
}

impl HashEntry {
    pub fn new(hash: &str) -> Self {
        Self {
            hash: hash.to_string(),
            qr_num: None,
            filename: None,
            compressed: None,
            fragments: HashMap::new(),
        }
    }

    /// 不足しているフラグメントの番号を返す（1オリジン）
    pub fn missing_indices(&self) -> Vec<usize> {
        if let Some(total) = self.qr_num {
            (1..=total)
                .filter(|i| !self.fragments.contains_key(i))
                .collect()
        } else {
            vec![] // qr_numが不明な場合は判断不能
        }
    }

    pub fn is_complete(&self) -> bool {
        self.qr_num.is_some() && self.missing_indices().is_empty()
    }
}

/// テキスト行のリストからフラグメントを解析し、HashEntryのマップを返す
pub fn parse_lines(lines: &[&str]) -> HashMap<String, HashEntry> {
    let mut entries: HashMap<String, HashEntry> = HashMap::new();

    for line in lines {
        if let Some(frag) = parse_fragment_line(line) {
            let entry = entries
                .entry(frag.hash.clone())
                .or_insert_with(|| HashEntry::new(&frag.hash));

            // 1番目のフラグメント（1オリジン）からメタ情報を抽出
            if frag.index == 1 {
                if let Some((qr_num, filename, compressed)) = extract_meta(&frag.chunk) {
                    entry.qr_num = Some(qr_num);
                    entry.filename = Some(filename);
                    entry.compressed = Some(compressed);
                }
            }

            entry.fragments.insert(frag.index, frag.chunk);
        }
    }

    entries
}

/// 1行からフラグメントを抽出する。行中に複数形式が混在しても最初にマッチしたものを返す。
fn parse_fragment_line(line: &str) -> Option<ParsedFragment> {
    // パターン: <8桁hex>:<3桁数字>:<残り>
    let re_find = |s: &str| -> Option<(usize, usize)> {
        let bytes = s.as_bytes();
        'outer: for i in 0..s.len() {
            // hash 8文字チェック
            if i + 8 >= s.len() { break; }
            for j in 0..8 {
                if !bytes[i + j].is_ascii_hexdigit() { continue 'outer; }
            }
            if bytes[i + 8] != b':' { continue; }
            // index 3文字チェック
            if i + 12 >= s.len() { break; }
            for j in 0..3 {
                if !bytes[i + 9 + j].is_ascii_digit() { continue 'outer; }
            }
            if bytes[i + 12] != b':' { continue; }
            return Some((i, i + 13));
        }
        None
    };

    let (start, data_start) = re_find(line)?;
    let hash = line[start..start + 8].to_string();
    let index: usize = line[start + 9..start + 12].parse().ok()?;
    let chunk = line[data_start..].to_string();

    Some(ParsedFragment { hash, index, chunk })
}

/// chunk の先頭部分（0番目フラグメント）からメタ情報を抽出
/// qr_data = "NNN:<encoded_filename>:<compress_flag>:<b64data>"
fn extract_meta(chunk: &str) -> Option<(usize, String, bool)> {
    // chunk は qr_data の先頭から。0番目なら "NNN:..." から始まる
    let qr_num: usize = chunk[..3].parse().ok()?;
    if chunk.as_bytes().get(3) != Some(&b':') {
        return None;
    }
    let rest = &chunk[4..]; // "<encoded_filename>:<compress_flag>:<b64data>"

    let colon1 = rest.find(':')?;
    let encoded_filename = &rest[..colon1];
    let rest2 = &rest[colon1 + 1..];

    let colon2 = rest2.find(':')?;
    let compress_flag = &rest2[..colon2];

    let filename = percent_decode_str(encoded_filename)
        .decode_utf8()
        .ok()?
        .to_string();
    let compressed = compress_flag == "xv";

    Some((qr_num, filename, compressed))
}

/// HashEntry が完全な場合にデータを復元する
pub fn reconstruct(entry: &HashEntry) -> Result<Vec<u8>, String> {
    let qr_num = entry.qr_num.ok_or("qr_num が不明です")?;

    // フラグメントを結合して qr_data を復元
    let mut qr_data = String::new();
    for i in 1..=qr_num {
        let chunk = entry
            .fragments
            .get(&i)
            .ok_or(format!("フラグメント {} が不足しています", i))?;
        qr_data.push_str(chunk);
    }

    // qr_data = "NNN:<encoded_filename>:<compress_flag>:<b64data>"
    let colon1 = qr_data.find(':').ok_or("フォーマットエラー")?;
    let rest = &qr_data[colon1 + 1..];
    let colon2 = rest.find(':').ok_or("フォーマットエラー")?;
    let rest2 = &rest[colon2 + 1..];
    let colon3 = rest2.find(':').ok_or("フォーマットエラー")?;
    let b64 = &rest2[colon3 + 1..];

    // Base64デコード
    let decoded = STANDARD
        .decode(b64)
        .map_err(|e| format!("Base64デコードエラー: {}", e))?;

    // 解凍
    if entry.compressed == Some(true) {
        decompress_xz(&decoded)
    } else {
        Ok(decoded)
    }
}

fn decompress_xz(data: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Read;
    use xz2::read::XzDecoder;

    let mut decoder = XzDecoder::new(data);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| format!("xz解凍エラー: {}", e))?;
    Ok(out)
}
