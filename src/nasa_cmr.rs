use super::errors::Result;
use super::models::BoundingBox;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CmrResponse {
    feed: CmrFeed,
}

#[derive(Deserialize, Debug)]
struct CmrFeed {
    entry: Vec<CmrEntry>,
}

#[derive(Deserialize, Debug)]
struct CmrEntry {
    title: String,
    links: Option<Vec<CmrLink>>,
}

#[derive(Deserialize, Debug)]
struct CmrLink {
    href: String,
    rel: String,
}

pub struct CmrClient {
    http_client: Client,
}

impl CmrClient {
    pub fn new() -> Self {
        Self {
            http_client: Client::builder().cookie_store(true).build().unwrap(),
        }
    }

    /// Ищет ссылки на файлы (гранулы) по заданым критериям
    pub async fn fetch_granules(
        &self,
        short_name: &str,
        bbox: &BoundingBox,
        temporal: &str,
    ) -> Result<Vec<(String, String)>> {
        // Формируем URL к поисковому движку NASA
        let url = format!(
            "https://nasa.gov?\
                 short_name={}&version=061&bounding_box={}&temporal={}",
            short_name,
            bbox.to_cmr_string(),
            temporal
        );

        // Делаем асинхронный HTTP-запрос
        let response: CmrResponse = self
            .http_client
            .get(&url)
            .header("User-Agent", "HydrogenGateway/1.0")
            .send()
            .await?
            .json()
            .await?;

        let mut target_links = Vec::new();

        // Парсим ответ и вытягиваем прямые ссылки на скачивание
        for entry in response.feed.entry {
            if let Some(links) = entry.links {
                // "data#" в поле rel означает, что это ссылка на сам файл данных, а не на картинку-превью
                if let Some(data_link) = links.iter().find(|l| l.rel.contains("data#")) {
                    target_links.push((entry.title.clone(), data_link.href.clone()));
                }
            }
        }

        Ok(target_links)
    }
}
