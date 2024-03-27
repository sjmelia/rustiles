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

        let result = MbTiles {
            connection: connection,
        };

        Ok(result)
    }

    pub fn get_tile(self, zoom_level: i64, tile_column: i64, tile_row: i64) -> Result<Vec<u8>> {
        let query = "SELECT * FROM tiles WHERE zoom_level = ? AND tile_column = ? AND tile_row = ?";
        let mut statement = self.connection.prepare(query)?;
        statement.bind((1, zoom_level))?;
        statement.bind((2, tile_column))?;
        statement.bind((3, tile_row))?;

        while let Ok(Row) = statement.next() {
            let tile_data = statement.read::<Vec<u8>, _>("tile_data")?;
            let mut decoder = GzDecoder::new(tile_data.as_slice());
            let mut buffer = Vec::new();
            decoder.read_to_end(&mut buffer).unwrap();
            return Ok(buffer);
        }

        return Err(anyhow!("No tile found"));
    }
}
