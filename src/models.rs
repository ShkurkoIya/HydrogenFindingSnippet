use geo_types::{Coord, LineString, Polygon};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct BoundingBox {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

impl BoundingBox {
    pub fn new(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> Self {
        Self {
            min_lon,
            min_lat,
            max_lon,
            max_lat,
        }
    }

    pub fn to_cmr_string(&self) -> String {
        format!(
            "{},{},{},{}",
            self.min_lon, self.min_lat, self.max_lon, self.max_lat
        )
    }
}

/// Метаданные привязки растра к планете Земля
#[derive(Debug, Clone)]
pub struct GeoTransform {
    pub x_origin: f64,
    pub pixel_width: f64,
    pub x_rotation: f64,
    pub y_origin: f64,
    pub y_rotation: f64,
    pub pixel_height: f64,
}

impl GeoTransform {
    /// Пример для спутника полигон Мура (Австралия) тут известная аномалия водорода
    pub fn mock_moora() -> Self {
        Self {
            x_origin: 115.5,
            pixel_width: 0.01,
            x_rotation: 0.0,
            y_origin: -30.0,
            y_rotation: 0.0,
            pixel_height: -0.01,
        }
    }

    ///  Переводит индекс пикселя [row, col] в географическую точку (центр пикселя)
    pub fn pixel_to_geo(&self, row: usize, col: usize) -> (f64, f64) {
        let lon = self.x_origin + (col as f64) * self.pixel_width + 0.5 * self.pixel_width;
        let lat = self.y_origin + (row as f64) * self.pixel_height + 0.5 * self.pixel_height;
        (lon, lat)
    }

    /// Превращает пиксель [row, col] в векторный четырехугольник (Polygon) для ГИС
    pub fn pixel_to_polygon(&self, row: usize, col: usize) -> Polygon<f64> {
        // Координаты углов пикселя
        let left_lon = self.x_origin + (col as f64) * self.pixel_width;
        let right_lon = left_lon + self.pixel_width;
        let top_lat = self.y_origin + (row as f64) * self.pixel_height;
        let bottom_lat = top_lat + self.pixel_height;

        // Замыкающий контур: Левый верхний -> Правый-Верхний -> Парвый-Нижний -> Левый-Нижний ->
        // Левый-Верхний
        let exterior = LineString::new(vec![
            Coord {
                x: left_lon,
                y: top_lat,
            },
            Coord {
                x: right_lon,
                y: top_lat,
            },
            Coord {
                x: right_lon,
                y: bottom_lat,
            },
            Coord {
                x: left_lon,
                y: top_lat,
            },
        ]);

        Polygon::new(exterior, vec![])
    }
}

/// Финальный векторный объект аномалии, готовы для сериализации в GeoJSON или записи в БД
#[derive(Debug)]
pub struct HydrogenAnomaly {
    pub center_pixel: (usize, usize),
    pub z_score: f32,
    pub value: f32,
    pub description: String,
    pub geometry: Polygon<f64>,
}
