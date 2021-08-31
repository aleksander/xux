#![feature(io_error_more)]

use image::ImageBuffer;
use image::Rgb;
use image::DynamicImage::ImageRgb8;
use std::path::PathBuf;
use std::fs::create_dir_all;
use std::io;
use std::error::Error;

// TODO grid_to_png(..., Mapper::first())
pub fn tiles_to_png(login: &str, name: &str, timestamp: &str, x: i32, y: i32, tiles: &[u8], palette: &[[u8;4]] /* TODO z: &[i16] */) -> Result<(), Box<dyn Error>> {
    let mut path = PathBuf::new();
    path.push(login);
    path.push(name);
    path.push(timestamp);

    if path.exists() {
        if ! path.is_dir() {
            return Err(Box::new(io::Error::from(io::ErrorKind::NotADirectory)));
        }
    } else {
        create_dir_all(&path)?;
    }

    path.push(format!("{} {}.png", x, y));

    let mut img = ImageBuffer::new(100, 100);
    for y in 0..100 {
        for x in 0..100 {
            let tile = tiles[y * 100 + x];
            let rgb = palette[tile as usize];
            img.put_pixel(x as u32, y as u32, Rgb([rgb[0], rgb[1], rgb[2]]));
        }
    }
    Ok(ImageRgb8(img).save(path)?)
}
