use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use sqlite::Connection;
use sqlite::State::Row;
use std::io::Read;

pub struct MbTiles {
    connection: Connection,
}

impl MbTiles {
    pub fn open(filename: &str) -> Result<MbTiles> {
        let connection = sqlite::open(filename)?;

        {
            let query = "SELECT value FROM metadata WHERE name = 'format'";
            let mut statement = connection.prepare(query)?;
            while let Ok(sqlite::State::Row) = statement.next() {
                let value = statement.read::<String, _>("value")?;
                if value != "pbf" {
                    return Err(anyhow!(
                        "Unexpected format {} - only pbf is supported",
                        value
                    ));
                }
            }
        }

        Ok(MbTiles { connection })
    }

    pub fn get_tile(&self, zoom_level: i64, tile_column: i64, tile_row: i64) -> Result<Vec<u8>> {
        let query =
            "SELECT tile_data FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ?";
        let mut statement = self.connection.prepare(query)?;
        statement.bind((1, zoom_level))?;
        statement.bind((2, tile_column))?;
        statement.bind((3, tile_row))?;

        while let Ok(Row) = statement.next() {
            let tile_data = statement.read::<Vec<u8>, _>("tile_data")?;
            if tile_data.starts_with(&[0x1f, 0x8b]) {
                let mut decoder = GzDecoder::new(tile_data.as_slice());
                let mut buffer = Vec::new();
                decoder.read_to_end(&mut buffer)?;
                return Ok(buffer);
            }

            return Ok(tile_data);
        }

        Err(anyhow!(
            "No tile found for z/x/y = {}/{}/{}",
            zoom_level,
            tile_column,
            tile_row
        ))
    }

    pub fn get_tile_xyz(&self, zoom_level: i64, tile_column: i64, y_xyz: i64) -> Result<Vec<u8>> {
        let max_row = (1_i64 << zoom_level) - 1;
        let y_tms = max_row - y_xyz;
        self.get_tile(zoom_level, tile_column, y_tms)
    }
}
