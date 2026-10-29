use crate::errors::GatewayError;

use super::errors::Result;
use super::models::BoundingBox;
use reqwest::{Client, header::ACCEPT, header::USER_AGENT};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CmrJsonResponse {
    items: Vec<CmrItem>,
}

#[derive(Deserialize, Debug)]
struct CmrItem {
    title: String,
    links: Option<Vec<CmrLink>>,
}

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
        let url = "https://cmr.earthdata.nasa.gov/search/granules.json";

        let params = [
            ("short_name", short_name),
            ("version", "061"),
            ("bounding_box", &bbox.to_cmr_string()),
            ("temporal", temporal),
        ];

        let request_builder = self
            .http_client
            .get(url)
            .query(&params)
            .header(USER_AGENT, "HydrogenGateway/1.0")
            .header(ACCEPT, "application/json");

        let response: CmrResponse = request_builder.send().await?.json().await?;

        let mut target_links = Vec::new();

        for entry in response.feed.entry {
            if let Some(links) = entry.links {
                // Ищем ссылку на скачивание (кликая по rel, содержащему "data#")
                if let Some(data_link) = links.iter().find(|l| l.rel.contains("data#")) {
                    target_links.push((entry.title.clone(), data_link.href.clone()));
                }
            }
        }

        Ok(target_links)
    }
}
