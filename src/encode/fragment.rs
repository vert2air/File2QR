use crate::encode::EcLevel;
use image::{GrayImage, Luma};
use qrcodegen::{QrCode, QrCodeEcc, QrSegment, Version};

/// QRコードをビットマップから直接、指定scaleで正方形ピクセル画像として生成
/// qrcodegen使用、Byteモード強制
pub fn generate_qr_image(
    data: &str,
    ec_level: EcLevel,
    scale: u32,
) -> Result<GrayImage, String> {
    let qr_ec = match ec_level {
        EcLevel::L => QrCodeEcc::Low,
        EcLevel::M => QrCodeEcc::Medium,
        EcLevel::Q => QrCodeEcc::Quartile,
        EcLevel::H => QrCodeEcc::High,
    };

    // Byteモードのセグメントを明示的に作成
    let seg = QrSegment::make_bytes(data.as_bytes());

    // QRコード生成（Byteモードのみ使用）
    let code = QrCode::encode_segments_advanced(
        &[seg],
        qr_ec,
        Version::new(1),  // 最小Version 1
        Version::new(40), // 最大Version 40
        None,             // マスクパターン自動選択
        true,             // 最適化有効
    )
    .map_err(|e| format!("QRコード生成エラー: {:?}", e))?;

    // QRコードのサイズを取得
    let width = code.size() as usize;

    // quiet_zone (余白) を追加 - 通常4モジュール分
    let quiet_zone = 4;
    let total_width = width + quiet_zone * 2;

    // scale倍の画像サイズ
    let img_width = total_width * scale as usize;
    let img_height = total_width * scale as usize;

    let mut img = GrayImage::new(img_width as u32, img_height as u32);

    // 全体を白で初期化
    for pixel in img.pixels_mut() {
        *pixel = Luma([255u8]);
    }

    // QRコードのビットマップを描画
    for y in 0..width {
        for x in 0..width {
            if code.get_module(x as i32, y as i32) {
                // 黒モジュール - scale x scale の正方形を描画
                let px_start = (x + quiet_zone) * scale as usize;
                let py_start = (y + quiet_zone) * scale as usize;

                for dy in 0..scale as usize {
                    for dx in 0..scale as usize {
                        let px = px_start + dx;
                        let py = py_start + dy;
                        if px < img_width && py < img_height {
                            img.put_pixel(px as u32, py as u32, Luma([0u8]));
                        }
                    }
                }
            }
        }
    }

    Ok(img)
}

/// egui で表示するための ColorImage に変換
pub fn to_egui_image(img: &GrayImage) -> egui::ColorImage {
    let (w, h) = img.dimensions();
    let pixels: Vec<egui::Color32> = img
        .pixels()
        .map(|p| {
            let v = p[0];
            egui::Color32::from_gray(v)
        })
        .collect();
    egui::ColorImage { size: [w as usize, h as usize], pixels }
}
