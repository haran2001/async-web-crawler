// src/main.rs

mod crawler;
mod robots_txt;
mod utils;

use crawler::Crawler;
use robots_txt::RobotsTxt;
use utils::fetch_robots_txt;

use reqwest::Client;
use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use url::Url;

// Constants
const MAX_CONCURRENT_REQUESTS: usize = 10;
const MAX_CRAWL_DEPTH: usize = 3;
const USER_AGENT: &str = "MyRustCrawler/1.0";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger
    env_logger::init();

    // Initialize the HTTP client with the custom user-agent
    let client = Client::builder().user_agent(USER_AGENT).build()?;

    // Define the base URL to start crawling
    let base_url = Url::parse("https://www.youtube.com/watch?v=TgmTsa3rFU0&t=1167s")?;

    // Fetch and parse robots.txt
    let robots_content = fetch_robots_txt(&client, &base_url).await?;
    let mut robots_parser = RobotsTxt::default();
    robots_parser.parse(&robots_content);

    // Initialize concurrency controls
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let visited = Arc::new(Mutex::new(HashSet::new()));

    // Create the crawler instance
    let crawler = Crawler {
        visited: visited.clone(),
        client: client.clone(),
        parser: robots_parser,
        semaphore: semaphore.clone(),
        max_depth: MAX_CRAWL_DEPTH,
        user_agent: USER_AGENT.to_string(),
    };

    // Start crawling with the base URL and depth 0
    crawler.crawl(base_url, 0).await;

    Ok(())
}
