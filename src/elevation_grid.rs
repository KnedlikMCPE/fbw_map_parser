pub struct ElevationGrid {
    pub southwest_latitude: f32,
    pub southwest_longitude: f32,
    pub northeast_latitude: f32,
    pub northeast_longitude: f32,
    pub rows: usize,
    pub columns: usize,
    pub elevation_map: Vec<i16>
}
impl ElevationGrid {
    pub fn new(southwest_latitude: f32, southwest_longitude: f32, northeast_latitude: f32, northeast_longitude: f32, rows: usize, columns: usize) -> Self {
        let map = vec![0; rows * columns];
        Self {
            southwest_latitude,
            southwest_longitude,
            northeast_latitude,
            northeast_longitude,
            rows,
            columns,
            elevation_map: map
        }
    }

    pub fn world_to_grid_indices(&mut self, latitude: f32, longitude: f32) -> (f32, f32) {
        let lat_range = self.northeast_latitude - self.southwest_latitude;
        let lat_delta = latitude - self.southwest_latitude;
        let row = f32::min(self.rows as f32 - f32::floor((lat_delta / lat_range) * self.rows as f32), self.rows as f32) - 1.0;

        let long_range = self.northeast_longitude - self.southwest_longitude;
        let long_delta = longitude - self.southwest_longitude;
        let column = f32::min(f32::floor((long_delta / long_range) * self.columns as f32), self.columns as f32 - 1.0);

        (row, column)
    }
}