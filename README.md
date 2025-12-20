# Bing Webmaster API Client

A Rust client library for the [Bing Webmaster API](https://learn.microsoft.com/en-us/bingwebmaster/), providing access to all methods from Microsoft's Bing Webmaster Tools.

## Features

- **Complete API Coverage**: Implements all methods from the IWebmasterApi interface
- **Type-Safe**: Strongly typed request and response structures using serde
- **Async/Await**: Built with async/await support using reqwest and tokio
- **Middleware Support**: Built on reqwest-middleware for extensibility (retry, logging, etc.)
- **Tracing Support**: Instrumented with tracing for observability
- **Error Handling**: Uses anyhow for comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bing-webmaster-api = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use bing_webmaster_api::{WebmasterApiClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = WebmasterApiClient::new(
        "your-api-key".to_string(),
        None // Use default Bing API URL
    );

    // Add a site
    client.add_site("https://example.com").await?;

    // Submit a URL
    client.submit_url("https://example.com", "https://example.com/page1").await?;

    // Get crawl issues
    let issues = client.get_crawl_issues("https://example.com").await?;
    println!("Found {} URLs with crawl issues", issues.len());

    Ok(())
}
```

## Using with Middleware

The client supports `reqwest-middleware` for adding custom middleware like retry policies, logging, etc:

```rust
use bing_webmaster_api::{WebmasterApiClient, ClientBuilder, Result};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client builder with custom middleware
    let client_builder = ClientBuilder::new(Client::new())
        // Add middleware here
        // .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        ;

    let client = WebmasterApiClient::with_middleware(
        "your-api-key".to_string(),
        None,
        client_builder
    );

    // Use the client as normal
    client.add_site("https://example.com").await?;

    Ok(())
}
```

## Authentication

You'll need an API key from Bing Webmaster Tools:

1. Sign in to [Bing Webmaster Tools](https://www.bing.com/webmasters/)
2. Go to Settings > API Access
3. Generate an API key
4. Use the key with this client

## Rate Limits

- URL Submission: Up to 10,000 URLs per day
- Content Submission: Up to 10MB payload per request
- Check your quota with `get_content_submission_quota()`

## Examples

See the `examples/` directory for more usage examples.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.