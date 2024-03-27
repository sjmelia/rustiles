use mvt_reader::Reader;

use log::trace;

mod mbtiles;
use crate::mbtiles::MbTiles;
mod renderer;
use crate::renderer::render_tile;

fn main() {
    env_logger::init();

    trace!("main::enter");
    /*
        let zoom_level = 14;
        let tile_column = 8580; //8568;
        let tile_row = 10646; //10637;
    */
    let zoom_level = 14;
    let tile_column = 8568;
    let tile_row = 10637;
    let mbtiles = MbTiles::open("C:\\Users\\steve\\zurich_switzerland.mbtiles")
        .expect("Could not open mbtiles file");
    let tile = mbtiles.get_tile(zoom_level, tile_column, tile_row).expect("Could not find tile");
    let reader = Reader::new(tile).expect("Could not read MVT data");
    let pixmap = render_tile(reader).expect("Could not render tile");
    pixmap.save_png("image.png").expect("Could not save file");
}
