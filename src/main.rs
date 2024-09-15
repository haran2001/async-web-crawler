// src/main.rs

mod crawler;
mod robots_txt;
mod utils;

use crawler::Crawler;
use robots_txt::RobotsTxt;
use utils::fetch_robots_txt;

use reqwest::Client;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use url::Url;

// Constants
const MAX_CONCURRENT_REQUESTS: usize = 10;
const MAX_CRAWL_DEPTH: usize = 3;
const USER_AGENT: &str = "MyRustCrawler/1.0";

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use env_logger;
use log::{error, info};
use serde::Deserialize;

const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rust Web Crawler</title>
</head>
<body>
    <h1>Rust Web Crawler</h1>
    <form action="/crawl" method="post">
        <label for="url">Enter URL to crawl:</label><br><br>
        <input type="text" id="url" name="url" size="50" required><br><br>
        <input type="submit" value="Crawl">
    </form>
</body>
</html>
"#;

#[derive(Deserialize)]
struct CrawlRequest {
    url: String,
}

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(INDEX_HTML)
}

async fn crawl_handler(
    data: web::Data<Arc<Crawler>>,
    form: web::Form<CrawlRequest>,
) -> impl Responder {
    let url_input = &form.url;

    // Parse the input URL
    let base_url = match Url::parse(url_input) {
        Ok(url) => url,
        Err(e) => {
            error!("Invalid URL provided: {}", e);
            return HttpResponse::BadRequest().body("Invalid URL provided.");
        }
    };

    // Start crawling
    info!("Starting crawl for URL: {}", base_url);

    data.crawl(base_url, 0).await;

    HttpResponse::Ok().body("Crawling initiated. Check the server logs for details.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the logger
    env_logger::init();

    // Initialize the HTTP client with the custom user-agent
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .expect("Failed to build HTTP client");

    // Define the base URL to start crawling (will be replaced by user input)
    let base_url = Url::parse("https://www.example.com").expect("Invalid base URL");

    // Fetch and parse robots.txt
    let robots_content = fetch_robots_txt(&client, &base_url)
        .await
        .expect("Failed to fetch robots.txt");
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

    let crawler_data = web::Data::new(Arc::new(crawler));

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(crawler_data.clone())
            .route("/", web::get().to(index))
            .route("/crawl", web::post().to(crawl_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
