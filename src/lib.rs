pub mod elevation_grid;
use elevation_grid::ElevationGrid;
use zune_inflate::DeflateDecoder;

#[derive(Clone)]
pub struct Tile {
    pub rows: u16,
    pub columns: u16,
    pub southwest_latitude: i8,
    pub southwest_longitude: i16,
    pub size: u32,
    pub data_offset: usize,
    pub compressed_data: &'static [u8],
    parent: *mut Map
}
impl Tile {
    pub fn from_bytes(parent: &mut Map, bytes: &'static [u8], start_offset: usize) -> Self {
        let size = u32::from_le_bytes([bytes[start_offset + 7], bytes[start_offset + 8], bytes[start_offset + 9], bytes[start_offset + 10]]);
        let data_offset = start_offset + 11;
        let new_data: &'static [u8] = &bytes[data_offset..(data_offset + size as usize)];
        Self {
            rows: u16::from_le_bytes([bytes[start_offset], bytes[start_offset + 1]]),
            columns: u16::from_le_bytes([bytes[start_offset + 2], bytes[start_offset + 3]]),
            southwest_latitude: i8::from_le_bytes([bytes[start_offset + 4]]),
            southwest_longitude: i16::from_le_bytes([bytes[start_offset + 5], bytes[start_offset + 6]]),
            size,
            data_offset,
            compressed_data: new_data,
            parent: parent
        }
    }

    pub fn load_elevation_grid(&mut self) -> ElevationGrid {
        let northeast_latitude = self.southwest_latitude as f32 + unsafe { self.parent.as_ref().unwrap().angular_steps_latitude } as f32;
        let northeast_longitude = self.southwest_longitude as f32 + unsafe { self.parent.as_ref().unwrap().angular_steps_longitude } as f32;
        let mut retval = ElevationGrid::new(self.southwest_latitude as f32, self.southwest_longitude as f32, northeast_latitude, northeast_longitude, self.rows as usize, self.columns as usize);

        let mut decoder = DeflateDecoder::new(self.compressed_data);
        let decompressed = decoder.decode_gzip().unwrap();

        let mut offset = 0;
        for row in 0..self.rows {
            for column in 0..self.columns {
                retval.elevation_map[row as usize * self.columns as usize + column as usize] = i16::from_le_bytes([decompressed[offset], decompressed[offset + 1]]);
                if retval.elevation_map[row as usize * self.columns as usize + column as usize] != -1 {
                    retval.elevation_map[row as usize * self.columns as usize + column as usize] = f32::round(retval.elevation_map[row as usize * self.columns as usize + column as usize] as f32 * 3.28084) as i16;
                }
                offset += 2;
            }
        }

        retval
    }
}

#[derive(Clone)]
pub struct Map {
    pub latitude_min: i16,
    pub latitude_max: i16,
    pub longitude_min: i16,
    pub longitude_max: i16,
    pub angular_steps_latitude: u8,
    pub angular_steps_longitude: u8,
    pub horizontal_resolution: f32,
    pub tiles: Vec<Tile>
}
impl Map {
    pub fn from_bytes(bytes: &'static [u8]) -> Self {
        let latitude_min = i16::from_le_bytes([bytes[0], bytes[1]]);
        let latitude_max = i16::from_le_bytes([bytes[2], bytes[3]]);
        let longitude_min = i16::from_le_bytes([bytes[4], bytes[5]]);
        let longitude_max = i16::from_le_bytes([bytes[6], bytes[7]]);
        let angular_steps_latitude = bytes[8];
        let angular_steps_longitude = bytes[9];
        let horizontal_resolution = f32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]);
        let tiles: Vec<Tile> = Vec::new();
        let mut offset = 14;
        let mut this = Self {
            latitude_min,
            latitude_max,
            longitude_min,
            longitude_max,
            angular_steps_latitude,
            angular_steps_longitude,
            horizontal_resolution,
            tiles
        };

        while offset < bytes.len() {
            let tile = Tile::from_bytes(&mut this, bytes, offset);
            let size = tile.size as usize;
            this.tiles.push(tile);
            offset += 11 + size;
        }

        this
    }

    pub fn get_elevation_at(&mut self, latitude: f32, longitude: f32) -> i16 {
        let mut elevation = -1;
        for tile in &mut self.tiles {
            if latitude >= tile.southwest_latitude as f32 && latitude < tile.southwest_latitude as f32 + self.angular_steps_latitude as f32 && longitude >= tile.southwest_longitude as f32 && longitude < tile.southwest_longitude as f32 + self.angular_steps_longitude as f32 {
                let mut grid = tile.load_elevation_grid();
                let (row, column) = grid.world_to_grid_indices(latitude, longitude);
                elevation = grid.elevation_map[row as usize * grid.columns + column as usize];
                break;
            }
        }
        elevation
    }
}