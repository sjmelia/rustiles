use anyhow::Result;
use rustiles::render_tile_bytes_with_style;
use rustiles_style::Style;
use std::env;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "Usage: {} <tile.mvt> <style.json> <output.png>",
            args.first()
                .map(String::as_str)
                .unwrap_or("render_sample_tile")
        );
        std::process::exit(2);
    }

    let tile_data = fs::read(&args[1])?;
    let style_json = fs::read_to_string(&args[2])?;
    let style = Style::from_str(&style_json)?;

    let pixmap = render_tile_bytes_with_style(tile_data, &style)?;
    pixmap.save_png(&args[3])?;

    Ok(())
}
