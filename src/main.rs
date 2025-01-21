use std::time::SystemTime;

use image::{open, GrayImage, ImageBuffer, Luma, RgbaImage};
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;

fn main() {
    // génération d'image à partir de bruit de perlin
    let thresholds = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

    let img = generate_perlin_image(1080, 1920, thresholds);
    img.save("perlin.png").expect("Failed to save image");
    // chargement de l'image
    let img = open("input.png")
        .expect("Failed to open image")
        .into_rgba8();

    // grayscale
    let gray_img = convert_to_luminance(&img);

    // masque de luminance, permet de conserver la structure de l'image lors du tri.
    let (low_thresh, high_thresh) = (50, 150); //seuil de luminance pour le masque, varible permettant de contrôler le rendu final
    let mask = create_luminance_mask(&gray_img, low_thresh, high_thresh);

    mask.save("mask.png").expect("Failed to save image"); //-->FOR DEBUG
                                                          //tri des pixels selon le masque de luminance (blanc = 255->pixel a trier, noir = 0->ne pas trier), les pixels sont triés par bandes continues de pixels blancs.
    let sorted_img = sort_image_using_mask(&img, &mask);

    sorted_img.save("output.png").expect("Failed to save image");
}

fn generate_perlin_image(height: i32, width: i32, thresholds: Vec<f64>) -> RgbaImage {
    let mut rng = rand::thread_rng();
    let colors: Vec<[u8; 3]> = (0..thresholds.len())
        .map(|_| [rng.gen(), rng.gen(), rng.gen()])
        .collect();

    let perlin = Perlin::new(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32,
    );
    let mut img = ImageBuffer::new(width as u32, height as u32);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let value = perlin.get([
            x as f64 / (width as f64 * 0.2),
            y as f64 / (height as f64 * 0.2),
        ]);
        let index = thresholds
            .iter()
            .position(|&t| value < t)
            .unwrap_or(thresholds.len() - 1);
        let color = colors[index];
        *pixel = image::Rgba([color[0], color[1], color[2], 255]);
    }

    img
}

fn convert_to_luminance(img: &RgbaImage) -> GrayImage {
    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let pixel = img.get_pixel(x, y);
        let luminance =
            (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8;
        Luma([luminance])
    })
}

fn create_luminance_mask(img: &GrayImage, low: u8, high: u8) -> GrayImage {
    ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let luminance = img.get_pixel(x, y)[0];
        if luminance >= low && luminance <= high {
            Luma([255])
        } else {
            Luma([0])
        }
    })
}

fn sort_image_using_mask(orig_img: &RgbaImage, mask: &GrayImage) -> RgbaImage {
    let mut sorted_img = orig_img.clone();
    for y in 0..mask.height() {
        let mut row: Vec<[u8; 4]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for x in 0..mask.width() {
            if mask.get_pixel(x, y)[0] == 255 {
                row.push(orig_img.get_pixel(x, y).0);
                indices.push(x);
            } else {
                if !row.is_empty() {
                    row.sort_by_key(|&p| {
                        let luminance =
                            (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) as u8;
                        luminance
                    });
                    for (i, &x_pos) in indices.iter().enumerate() {
                        sorted_img.put_pixel(x_pos, y, image::Rgba(row[i]));
                    }
                    row.clear();
                    indices.clear();
                }
            }
        }
        if !row.is_empty() {
            row.sort_by_key(|&p| {
                let luminance =
                    (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) as u8;
                luminance
            });
            for (i, &x_pos) in indices.iter().enumerate() {
                sorted_img.put_pixel(x_pos, y, image::Rgba(row[i]));
            }
            row.clear();
            indices.clear();
        }
    }
    sorted_img
}
