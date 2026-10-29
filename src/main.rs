pub mod analytics;
pub mod errors;
pub mod models;
pub mod nasa_cmr;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("--- Starting Finding Hydrogen Anomalies ---");

    // Инициализируем тестовый полигон Мура (Западная Австралияс)
    let moora_bbox = models::BoundingBox::new(115.9, -30.7, 116.1, -30.5);
    let time_window = "2026-01-01T00:00:00Z,2026-01-15T23:59:59Z";

    // Поиск метаданных в NASA CMR API
    let cmr_client = nasa_cmr::CmrClient::new();
    println!("\n Request to NASA CMR API for product MOD11A1 (Soil Temperature)...");

    match cmr_client
        .fetch_granules("MOD11A1", &moora_bbox, time_window)
        .await
    {
        Ok(granules) => {
            println!(
                " ✅ Found sattelite files from NASA archive: {}",
                granules.len()
            );

            for (name, url) in granules.iter().take(2) {
                println!("  -> Shot: {}", name);
                println!("     Download link: {}", url);
            }
        }
        Err(e) => println!(" ❌ Ошибка запроса к NASA: {}", e),
    }

    // Математическая калибровка, детекция и векторизация
    println!("Initializing GeoTransform and raster");

    let transform = models::GeoTransform::mock_moora();

    // Якобы вырезали кусок из снимка
    let mut mock_raster = ndarray::Array2::<f32>::from_elem((8, 8), 24.0);

    mock_raster[[3, 4]] = 29.5;

    // Настраиваем детектор: искать все, что отклоняется более чем на 2.5 стандартных отклонения
    let detector = analytics::AnomalyDetector::new(2.5);

    println!("Running algorithm of Z-score filtration + vectorization...");

    let anomalies = detector.detect_anomalies(&mock_raster, &transform)?;

    println!("\n--- Results of Scanning ---");

    if anomalies.is_empty() {
        println!("Hydrogen anomalies not found");
    } else {
        println!("🔥 FOUND ANOMALY ZONES: {}", anomalies.len());
        for anomaly in anomalies {
            println!(
                "📍 Anomaly in pixels[Y:{}, X:{}]",
                anomaly.center_pixel.0, anomaly.center_pixel.1
            );
            println!(
                "   Value: {}°C (Z-Score: {:.2}) -> {}",
                anomaly.value, anomaly.z_score, anomaly.description
            );

            // Извлекаем вершины полигона, которые были рассчитаны модулем трансфигурации
            let exterior = anomaly.geometry.exterior();
            println!("   Polygon coordinates (WGS84):");
            for coord in exterior.coords() {
                println!(
                    "     -> Longitude (Lon): {:.4}, Lattitude(Lat): {:.4}",
                    coord.x, coord.y
                );
            }
        }
    }

    println!("Finishing pipeline!");

    Ok(())
}
