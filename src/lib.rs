//! # Bing Webmaster API Client
//!
//! This crate provides a Rust client for the [Bing Webmaster API](https://learn.microsoft.com/en-us/bingwebmaster/),
//! allowing you to interact with Bing Webmaster Tools programmatically.
//!
//! ## Features
//!
//! - **Complete API Coverage**: All non-obsolete methods from the IWebmasterApi interface
//! - **Type-Safe**: Strongly typed structures with serde support
//! - **Async/Await**: Built for modern async Rust applications
//! - **Middleware Support**: Built on reqwest-middleware for extensibility
//! - **Error Handling**: Comprehensive error handling with specific error types and anyhow integration
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use bing_webmaster_api::{BingWebmasterClient, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = BingWebmasterClient::new("your-api-key".to_string());
//!
//!     // Add a site
//!     client.add_site("https://example.com").await?;
//!
//!     // Submit URLs
//!     client.submit_url("https://example.com", "https://example.com/page1").await?;
//!
//!     // Get crawl issues
//!     let issues = client.get_crawl_issues("https://example.com").await?;
//!     println!("Found {} URLs with issues", issues.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## API Methods
//!
//! ### Site Management
//! - [`add_site`](BingWebmasterClient::add_site) - Add a new site
//! - [`verify_site`](BingWebmasterClient::verify_site) - Verify site ownership
//! - [`add_site_roles`](BingWebmasterClient::add_site_roles) - Manage user permissions
//!
//! ### URL Submission
//! - [`submit_url`](BingWebmasterClient::submit_url) - Submit single URL
//! - [`submit_url_batch`](BingWebmasterClient::submit_url_batch) - Submit multiple URLs
//! - [`submit_content`](BingWebmasterClient::submit_content) - Submit content with metadata
//!
//! ### Analytics
//! - [`get_crawl_issues`](BingWebmasterClient::get_crawl_issues) - Get crawl problems
//! - [`get_page_stats`](BingWebmasterClient::get_page_stats) - Get traffic statistics
//! - [`get_crawl_stats`](BingWebmasterClient::get_crawl_stats) - Get crawl statistics

pub mod client;
pub mod dto;
pub mod error;

pub use client::BingWebmasterClient;
pub use dto::*;
pub use error::{BingErrorCode, Result, WebmasterApiError};

// Re-export middleware types for convenience
pub use reqwest_middleware::{ClientBuilder, Middleware};
