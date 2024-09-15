// src/utils.rs

use reqwest::Client;
use scraper::{Html, Selector};
use std::error::Error;
use url::Url;

/// Fetches the robots.txt content from the given base URL
pub async fn fetch_robots_txt(client: &Client, base_url: &Url) -> Result<String, Box<dyn Error>> {
    let robots_url = base_url.join("/robots.txt")?;
    let response = client.get(robots_url).send().await?;

    if response.status().is_success() {
        let text = response.text().await?;
        Ok(text)
    } else {
        // If robots.txt is not found or inaccessible, assume no restrictions
        Ok(String::new())
    }
}

/// Extracts and resolves all links from the HTML content relative to the base URL
pub fn extract_links(base_url: &Url, html_content: &str) -> Vec<Url> {
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("a[href]").unwrap();
    let mut links = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(resolved_url) = base_url.join(href) {
                links.push(resolved_url);
            }
        }
    }

    links
}
