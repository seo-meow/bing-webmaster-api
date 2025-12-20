use bing_webmaster_api::{BingWebmasterClient, ClientBuilder, Result};
use reqwest::Client;
use reqwest_middleware::Middleware;

// Example custom middleware for logging requests
#[derive(Debug)]
pub struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut http::Extensions,
        next: reqwest_middleware::Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        println!("Making request to: {}", req.url());

        let response = next.run(req, extensions).await?;

        println!("Response status: {}", response.status());

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client builder with custom middleware
    let client_builder = ClientBuilder::new(Client::new()).with(LoggingMiddleware);

    let client = BingWebmasterClient::with_middleware(
        std::env::var("BING_WEBMASTER_API_KEY")
            .expect("BING_WEBMASTER_API_KEY environment variable must be set"),
        None,
        client_builder,
    );

    let site_url = "https://example.com";

    // The middleware will log all requests and responses
    println!("Adding site with middleware logging...");
    client.add_site(site_url).await?;

    println!("Submitting URL with middleware logging...");
    client
        .submit_url(site_url, "https://example.com/page1")
        .await?;

    println!("Getting crawl issues with middleware logging...");
    let issues = client.get_crawl_issues(site_url).await?;
    println!("Found {} URLs with crawl issues", issues.len());

    println!("Middleware example completed!");
    Ok(())
}
