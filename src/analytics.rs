use super::errors::{GatewayError, Result};
use super::models::{GeoTransform, HydrogenAnomaly};
use ndarray::Array2;

pub struct AnomalyDetector {
    z_threshold: f32,
}

impl AnomalyDetector {
    pub fn new(z_threshold: f32) -> Self {
        Self { z_threshold }
    }

    pub fn detect_anomalies(
        &self,
        matrix: &Array2<f32>,
        transform: &GeoTransform,
    ) -> Result<Vec<HydrogenAnomaly>> {
        if matrix.is_empty() {
            return Err(GatewayError::EmptyMatrix);
        }

        let mean = matrix.mean().ok_or(GatewayError::EmptyMatrix)?;
        let variance = matrix.fold(0.0, |acc, &x| acc + (x - mean).powi(2)) / (matrix.len() as f32);
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return Err(GatewayError::MathError(
                "Cannot calibrate: zero deviation".into(),
            ));
        }

        let mut anomalies = Vec::new();

        for ((row, col), &val) in matrix.indexed_iter() {
            let z_score = (val - mean) / std_dev;

            if z_score >= self.z_threshold {
                let geo_poly = transform.pixel_to_polygon(row, col);

                anomalies.push(HydrogenAnomaly {
                    center_pixel: (row, col),
                    z_score,
                    value: val,
                    description: format!("Thermal scar (+{:.2}°C)", val - mean),
                    geometry: geo_poly,
                });
            }
        }

        Ok(anomalies)
    }
}
