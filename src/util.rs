// TODO grid_to_png(..., Mapper::first())
pub fn grid_to_png(x: i32, y: i32, t: &[u8], z: &[i16]) {

    use std::fs::File;
    use image::ImageBuffer;
    use image::Rgb;
    use image::ImageRgb8;
    use image::PNG;
    use shift_to_unsigned::ShiftToUnsigned;

    let mut f = File::create(format!("{} {}.png", x, y)).expect("grid2png.file.create");
    let mut img = ImageBuffer::new(100, 100);
    for y in 0..100 {
        for x in 0..100 {
            let t = t[y * 100 + x];
            let z = z[y * 100 + x];
            let z = z.shift_to_unsigned();
            let h = (z >> 8) as u8;
            let l = z as u8;
            let mut r = 0;
            r |= (t >> 0) & 1;
            r <<= 1;
            r |= (t >> 3) & 1;
            r <<= 1;
            r |= (t >> 6) & 1;
            r <<= 1;
            r |= (h >> 4) & 1;
            r <<= 1;
            r |= (h >> 1) & 1;
            r <<= 1;
            r |= (l >> 6) & 1;
            r <<= 1;
            r |= (l >> 3) & 1;
            r <<= 1;
            r |= (l >> 0) & 1;
            let mut g = 0;
            g |= (t >> 1) & 1;
            g <<= 1;
            g |= (t >> 4) & 1;
            g <<= 1;
            g |= (t >> 7) & 1;
            g <<= 1;
            g |= (h >> 5) & 1;
            g <<= 1;
            g |= (h >> 2) & 1;
            g <<= 1;
            g |= (l >> 7) & 1;
            g <<= 1;
            g |= (l >> 4) & 1;
            g <<= 1;
            g |= (l >> 1) & 1;
            let mut b = 0;
            b |= (t >> 2) & 1;
            b <<= 1;
            b |= (t >> 5) & 1;
            b <<= 1;
            b |= (h >> 7) & 1;
            b <<= 1;
            b |= (h >> 6) & 1;
            b <<= 1;
            b |= (h >> 3) & 1;
            b <<= 1;
            b |= (h >> 0) & 1;
            b <<= 1;
            b |= (l >> 5) & 1;
            b <<= 1;
            b |= (l >> 2) & 1;
            // let mut r = 0;
            // r |= (t >> 2) & 1; r <<= 1;
            // r |= (t >> 3) & 1; r <<= 1;
            // r |= (h >> 7) & 1; r <<= 1;
            // r |= (h >> 6) & 1; r <<= 1;
            // r |= (h >> 1) & 1; r <<= 1;
            // r |= (h >> 0) & 1; r <<= 1;
            // r |= (l >> 3) & 1; r <<= 1;
            // r |= (l >> 2) & 1;
            // let mut g = 0;
            // g |= (t >> 1) & 1; g <<= 1;
            // g |= (t >> 4) & 1; g <<= 1;
            // g |= (t >> 7) & 1; g <<= 1;
            // g |= (h >> 5) & 1; g <<= 1;
            // g |= (h >> 2) & 1; g <<= 1;
            // g |= (l >> 7) & 1; g <<= 1;
            // g |= (l >> 4) & 1; g <<= 1;
            // g |= (l >> 1) & 1;
            // let mut b = 0;
            // b |= (t >> 0) & 1; b <<= 1;
            // b |= (t >> 5) & 1; b <<= 1;
            // b |= (t >> 6) & 1; b <<= 1;
            // b |= (h >> 4) & 1; b <<= 1;
            // b |= (h >> 3) & 1; b <<= 1;
            // b |= (l >> 6) & 1; b <<= 1;
            // b |= (l >> 5) & 1; b <<= 1;
            // b |= (l >> 2) & 1;
            //
            img.put_pixel(x as u32, y as u32, Rgb([g, r, b /* t,h,l */]));
        }
    }
    ImageRgb8(img).save(&mut f, PNG).expect("grid2png.image.save");
}
