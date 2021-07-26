use crate::Result;
use image::ImageBuffer;
use image::Rgb;
use image::DynamicImage::ImageRgb8;
use std::path::PathBuf;
use std::fs::create_dir_all;
use failure::format_err;

// TODO grid_to_png(..., Mapper::first())
pub fn grid_to_png(login: &str, name: &str, timestamp: &str, x: i32, y: i32, t: &[u8] /* TODO z: &[i16] */) -> Result<()> {
    let mut path = PathBuf::new();
    path.push(login);
    path.push(name);
    path.push(timestamp);

    if path.exists() {
        if ! path.is_dir() {
            return Err(format_err!("\"{}\" is not a dir", path.display()));
        }
    } else {
        create_dir_all(&path)?;
    }

    path.push(format!("{} {}.png", x, y));

    let mut img = ImageBuffer::new(100, 100);
    for y in 0..100 {
        for x in 0..100 {
            let t = t[y * 100 + x];
            //TODO get RGB from palette 'tile_colors.ron'
            let r = t;
            let g = 0;
            let b = 0;
            img.put_pixel(x as u32, y as u32, Rgb([g, r, b]));
        }
    }
    Ok(ImageRgb8(img).save(path)?)
}
