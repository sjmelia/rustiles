pub mod mbtiles;
pub mod renderer;

use anyhow::{anyhow, Result};
use mbtiles::MbTiles;
use mvt_reader::Reader;
use renderer::render_tile_with_style;
use rustiles_style::Style;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

pub fn render_tile_bytes_with_style(tile_data: Vec<u8>, style: &Style) -> Result<Pixmap> {
    let reader = Reader::new(tile_data).map_err(|_| anyhow!("Could not read MVT data"))?;
    render_tile_with_style(reader, style)
}

pub fn render_region_from_mbtiles(
    mbtiles: &MbTiles,
    style: &Style,
    zoom_level: i64,
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
) -> Result<Pixmap> {
    if min_x > max_x || min_y > max_y {
        return Err(anyhow!("invalid region bounds"));
    }

    let width_tiles = (max_x - min_x + 1) as u32;
    let height_tiles = (max_y - min_y + 1) as u32;

    let Some(mut canvas) = Pixmap::new(width_tiles * 4096, height_tiles * 4096) else {
        return Err(anyhow!("Could not allocate region canvas"));
    };

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let tile_data = mbtiles.get_tile_xyz(zoom_level, x, y)?;
            let tile = render_tile_bytes_with_style(tile_data, style)?;

            let dx = ((x - min_x) * 4096) as i32;
            let dy = ((y - min_y) * 4096) as i32;
            canvas.draw_pixmap(
                dx,
                dy,
                tile.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );
        }
    }

    Ok(canvas)
}
