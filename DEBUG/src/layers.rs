use image::{GrayImage, Luma};

/// Ergebnis der Ebenen-Analyse für jeden Pixel — 5 Layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelLayer {
    Black,      // sehr dunkel  (luma < black_threshold)
    Background, // dunkel       (luma < bg_threshold)
    Foreground, // mittel/hell
    Edge,       // Sobel-Kante
    White,      // sehr hell    (luma > white_threshold)
}

pub struct LayerResult {
    pub layers: Vec<PixelLayer>,
    pub detail: Vec<f32>,
    pub fine_detail: Vec<f32>,
    pub coarse_detail: Vec<f32>,
}

pub enum DetailScale {
    Fine,
    Medium,
    Coarse,
}

impl DetailScale {
    #[allow(dead_code)]
    pub fn threshold(&self) -> (f32, f32) {
        match self {
            DetailScale::Fine => (0.0, 0.15),
            DetailScale::Medium => (0.15, 0.35),
            DetailScale::Coarse => (0.35, 1.0),
        }
    }
}

/// Analysiert ein Graustufenbild und gibt pro Pixel die Ebene zurück.
/// Priorität: Edge > White > Black > Background > Foreground
pub fn detect_layers(
    gray: &GrayImage,
    bg_threshold: f32,
    _fg_threshold: f32,
    edge_threshold: f32,
) -> LayerResult {
    let black_threshold = bg_threshold * 0.45;
    let white_threshold = 1.0 - black_threshold;

    let w = gray.width() as usize;
    let h = gray.height() as usize;
    let pixels: Vec<f32> = gray.pixels().map(|Luma([v])| *v as f32 / 255.0).collect();
    let sobel = sobel_magnitude(&pixels, w, h);

    let fine_detail = laplacian_variance(&pixels, w, h, 1);
    let coarse_detail = laplacian_variance(&pixels, w, h, 2);

    let layers: Vec<PixelLayer> = pixels
        .iter()
        .zip(sobel.iter())
        .map(|(&luma, &mag)| {
            if mag > edge_threshold {
                PixelLayer::Edge
            } else if luma > white_threshold {
                PixelLayer::White
            } else if luma < black_threshold {
                PixelLayer::Black
            } else if luma < bg_threshold {
                PixelLayer::Background
            } else {
                PixelLayer::Foreground
            }
        })
        .collect();

    LayerResult {
        layers,
        detail: sobel,
        fine_detail,
        coarse_detail,
    }
}

fn sobel_magnitude(pixels: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let p = |dy: isize, dx: isize| -> f32 {
                pixels[((y as isize + dy) as usize) * w + ((x as isize + dx) as usize)]
            };
            let gx = -p(-1, -1) + p(-1, 1) - 2.0 * p(0, -1) + 2.0 * p(0, 1) - p(1, -1) + p(1, 1);
            let gy = -p(-1, -1) - 2.0 * p(-1, 0) - p(-1, 1) + p(1, -1) + 2.0 * p(1, 0) + p(1, 1);
            out[y * w + x] = (gx.abs() + gy.abs()) / 4.0;
        }
    }
    out
}

fn laplacian_variance(pixels: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    let r = radius as isize;

    for y in r..(h as isize - r) {
        for x in r..(w as isize - r) {
            let center = pixels[(y as usize) * w + (x as usize)];
            let mut sum = 0.0f32;
            let mut count = 0.0f32;

            for dy in -r..=r {
                for dx in -r..=r {
                    if dx != 0 || dy != 0 {
                        let neighbor = pixels[((y + dy) as usize) * w + ((x + dx) as usize)];
                        sum += (center - neighbor).powi(2);
                        count += 1.0;
                    }
                }
            }
            out[(y as usize) * w + (x as usize)] = (sum / count).sqrt();
        }
    }
    out
}

#[allow(dead_code)]
pub fn box_blur(pixels: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; w * h];
    let r = radius as isize;

    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0f32;
            let mut count = 0.0f32;

            for dy in -r..=r {
                for dx in -r..=r {
                    let ny = (y as isize + dy).clamp(0, h as isize - 1) as usize;
                    let nx = (x as isize + dx).clamp(0, w as isize - 1) as usize;
                    sum += pixels[ny * w + nx];
                    count += 1.0;
                }
            }
            output[y * w + x] = sum / count;
        }
    }
    output
}

#[allow(dead_code)]
pub fn difference_of_gaussians(
    pixels: &[f32],
    w: usize,
    h: usize,
    sigma1: f32,
    sigma2: f32,
) -> Vec<f32> {
    let r1 = (sigma1 * 1.5) as usize + 1;
    let r2 = (sigma2 * 1.5) as usize + 1;

    let blur1 = box_blur(pixels, w, h, r1);
    let blur2 = box_blur(pixels, w, h, r2);

    blur1.iter().zip(blur2.iter()).map(|(a, b)| a - b).collect()
}

pub fn get_detail_scale(detail_value: f32) -> DetailScale {
    if detail_value < 0.15 {
        DetailScale::Fine
    } else if detail_value < 0.35 {
        DetailScale::Medium
    } else {
        DetailScale::Coarse
    }
}
