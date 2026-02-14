use anyhow::{anyhow, Result};
use rustiles::mbtiles::MbTiles;
use rustiles::{render_region_from_mbtiles, render_tile_bytes_with_style};
use rustiles_style::Style;
use std::env;
use std::fs;

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 7 {
        eprintln!(
            "Usage:\n  {0} <mbtiles> <style.json> <z> <x> <y> <output.png>\n  {0} <mbtiles> <style.json> <z> <min_x> <min_y> <max_x> <max_y> <output.png>",
            args.first().map(String::as_str).unwrap_or("rustiles")
        );
        return Err(anyhow!("invalid args"));
    }

    let mbtiles = MbTiles::open(&args[1])?;
    let style_json = fs::read_to_string(&args[2])?;
    let style = Style::from_str(&style_json)?;
    let zoom = args[3].parse::<i64>()?;

    match args.len() {
        7 => {
            let x = args[4].parse::<i64>()?;
            let y = args[5].parse::<i64>()?;
            let output = &args[6];
            let tile_data = mbtiles.get_tile_xyz(zoom, x, y)?;
            let pixmap = render_tile_bytes_with_style(tile_data, &style)?;
            pixmap.save_png(output)?;
        }
        9 => {
            let min_x = args[4].parse::<i64>()?;
            let min_y = args[5].parse::<i64>()?;
            let max_x = args[6].parse::<i64>()?;
            let max_y = args[7].parse::<i64>()?;
            let output = &args[8];
            let pixmap =
                render_region_from_mbtiles(&mbtiles, &style, zoom, min_x, min_y, max_x, max_y)?;
            pixmap.save_png(output)?;
        }
        _ => return Err(anyhow!("invalid args")),
    }

    Ok(())
}
