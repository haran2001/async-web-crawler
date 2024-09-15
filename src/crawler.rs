// src/crawler.rs

use async_recursion::async_recursion;
use log::{error, info};
use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use url::Url;

use crate::robots_txt::RobotsTxt;
use crate::utils::extract_links;

/// Struct representing the web crawler
pub struct Crawler {
    pub visited: Arc<Mutex<HashSet<String>>>,
    pub client: Client,
    pub parser: RobotsTxt,
    pub semaphore: Arc<Semaphore>,
    pub max_depth: usize,
    pub user_agent: String,
}

impl Crawler {
    /// Recursively crawls the given URL up to the specified depth
    #[async_recursion]
    pub async fn crawl(&self, url: Url, depth: usize) {
        if depth > self.max_depth {
            return;
        }

        let url_str = url.to_string();

        // Check if URL is already visited
        {
            let mut visited_lock = self.visited.lock().await;
            if !visited_lock.insert(url_str.clone()) {
                info!("Already visited: {}", url);
                return;
            }
        }

        // Check if URL is allowed by robots.txt
        if !self.parser.is_allowed(&self.user_agent, url.path()) {
            info!("Disallowed by robots.txt: {}", url);
            return;
        }

        // Acquire a permit from the semaphore to limit concurrency
        let permit = self.semaphore.acquire().await.unwrap();

        // Fetch the page
        match self.client.get(url.clone()).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let body = resp.text().await.unwrap_or_default();
                    info!("Fetched: {}", url);

                    // Extract links and recursively crawl them
                    let links = extract_links(&url, &body);
                    for link in links {
                        // Only crawl links within the same domain
                        if link.domain() == url.domain() {
                            self.crawl(link, depth + 1).await;
                        }
                    }
                } else {
                    error!("Failed to fetch {}: {}", url, resp.status());
                }
            }
            Err(e) => {
                error!("Error fetching {}: {}", url, e);
            }
        }

        // Permit is automatically released here
        drop(permit);
    }
}
