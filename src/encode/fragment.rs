use qrcode::{QrCode, EcLevel as QrEcLevel};
use image::{GrayImage, Luma};
use crate::encode::EcLevel;

/// QRコードをビットマップから直接、指定scaleで正方形ピクセル画像として生成
pub fn generate_qr_image(
    data: &str,
    ec_level: EcLevel,
    scale: u32,
) -> Result<GrayImage, String> {
    let qr_ec = match ec_level {
        EcLevel::L => QrEcLevel::L,
        EcLevel::M => QrEcLevel::M,
        EcLevel::Q => QrEcLevel::Q,
        EcLevel::H => QrEcLevel::H,
    };

    let code = QrCode::with_error_correction_level(data.as_bytes(), qr_ec)
        .map_err(|e| format!("QRコード生成エラー: {}", e))?;

    // QRコードのビットマップを取得
    let width = code.width();
    let colors = code.to_colors();
    
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
            let module_idx = y * width + x;
            // Colorは黒(Dark)か白(Light)
            let is_dark = match colors[module_idx] {
                qrcode::Color::Dark => true,
                qrcode::Color::Light => false,
            };
            
            if is_dark {
                // 黒いモジュール: scale x scale の正方形を描画
                let img_x = (x + quiet_zone) * scale as usize;
                let img_y = (y + quiet_zone) * scale as usize;
                
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = (img_x + dx as usize) as u32;
                        let py = (img_y + dy as usize) as u32;
                        if px < img_width as u32 && py < img_height as u32 {
                            img.put_pixel(px, py, Luma([0u8]));
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
    egui::ColorImage {
        size: [w as usize, h as usize],
        pixels,
    }
}
