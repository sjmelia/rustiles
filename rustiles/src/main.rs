use mvt_reader::Reader;

use log::trace;

use rustiles;
use rustiles::mbtiles::MbTiles;
use rustiles::renderer::render_tile;

fn main() {
    env_logger::init();

    trace!("main::enter");
    
    /*
    let zoom_level = 14;
    let tile_column = 8568;
    let tile_row = 10637;
    let mbtiles = MbTiles::open("/home/steve/projects/naip/zurich_switzerland.mbtiles")
        .expect("Could not open mbtiles file");
    */
    ///*
    let zoom_level = 12;
    let tile_column = 656;
    let tile_row = 2670;
    let mbtiles = MbTiles::open("/home/steve/projects/naip/data/out/planet.mbtiles")
        .expect("Could not open mbtiles file");
    //*/
    let tile = mbtiles.get_tile(zoom_level, tile_column, tile_row).expect("Could not find tile");
    println!("Got tile!");
    let reader = Reader::new(tile).expect("Could not read MVT data");
    println!("Read tile!");
    let pixmap = render_tile(reader).expect("Could not render tile");
    println!("Rendered tile!");
    pixmap.save_png("image.png").expect("Could not save file");
}
