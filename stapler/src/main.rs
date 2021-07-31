use std::collections::BTreeMap;

//TODO background color for absent tiles #rrggbbaa
//TODO save as palette PNG (not rgba) because it's much smaller

fn main () {
    let input =  &std::env::args().nth(1).unwrap_or("./".into());
    let output = &std::env::args().nth(2).unwrap_or("out.png".into());
    //TODO read_dir()
    //      .expect()
    //      .filter(is_file)
    //      .filter(filename("{int} {int}.png"))
    //      .collect::Result<Path>()
    let mut tiles = BTreeMap::new();
    for entry in std::fs::read_dir(input).expect("read_dir") {
        let entry = entry.expect("path");
        if let Ok(ftype) = entry.file_type() {
            if ftype.is_file() {
                let fpath = entry.path();
                if let Some(ext) = fpath.extension() {
                    if ext == "png" {
                        if let Some(stem) = fpath.file_stem() {
                            if let Some(name) = stem.to_str() {
                                let xy = name.split(" ").collect::<Vec<&str>>();
                                if xy.len() == 2 {
                                    if let Ok(x) = xy[0].parse::<isize>() {
                                        if let Ok(y) = xy[1].parse::<isize>() {
                                            println!("{:?}: {} {}", fpath, x, y);
                                            tiles.insert((x, y), fpath.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if tiles.len() > 1 {
        let mut keys = tiles.keys();
        let &(x, y) = keys.next().expect("tiles.next");
        let mut minx = x;
        let mut miny = y;
        let mut maxx = x;
        let mut maxy = y;
        for &(x, y) in keys {
            if x < minx { minx = x; }
            if y < miny { miny = y; }
            if x > maxx { maxx = x; }
            if y > maxy { maxy = y; }
        }
        let dx = (maxx - minx + 1) as u32;
        let dy = (maxy - miny + 1) as u32;
        println!("{}x{}", dx, dy);
        for y in miny..=maxy {
            for x in minx..=maxx {
                if tiles.contains_key(&(x, y)) {
                    print!("0");
                } else {
                    print!(".");
                }
            }
            println!();
        }
        const TILE_SIZE: u32 = 100;
        let mut buf: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(dx * TILE_SIZE, dy * TILE_SIZE);
        for y in miny..=maxy {
            for x in minx..=maxx {
                if let Some(path) = tiles.get(&(x, y)) {
                    use image::GenericImage;
                    let tile = image::open(path).expect("image.open");
                    buf.copy_from(&tile, ((x - minx) as u32) * TILE_SIZE, ((y - miny) as u32) * TILE_SIZE).expect("unable to copy_from");
                }
            }
        }
        buf.save(output).expect("image.save");
    } else {
        println!("nothing to staple");
    }
}
