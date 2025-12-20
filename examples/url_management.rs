use bing_webmaster_api::{
    BingWebmasterClient, BlockReason, BlockedUrl, BlockedUrlEntityType, BlockedUrlRequestType,
    CountryRegionSettings, CountryRegionSettingsType, Result,
};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<()> {
    let client = BingWebmasterClient::new(
        std::env::var("BING_WEBMASTER_API_KEY")
            .expect("BING_WEBMASTER_API_KEY environment variable must be set"),
    );

    let site_url = "https://example.com";

    // Add URL normalization parameters
    println!("Adding query parameters to ignore...");
    client.add_query_parameter(site_url, "utm_source").await?;
    client.add_query_parameter(site_url, "utm_medium").await?;
    client.add_query_parameter(site_url, "utm_campaign").await?;

    // Get current query parameters
    println!("Getting current query parameters...");
    let params = client.get_query_parameters(site_url).await?;
    for param in &params {
        println!(
            "  Parameter: {} (enabled: {}, source: {})",
            param.parameter, param.is_enabled, param.source
        );
    }

    // Disable a parameter
    println!("Disabling utm_source parameter...");
    client
        .enable_disable_query_parameter(site_url, "utm_source", false)
        .await?;

    // Block specific URLs from search results
    println!("Adding blocked URLs...");
    let blocked_url = BlockedUrl {
        url: "https://example.com/admin".to_string(),
        date: Utc::now(),
        days_to_expire: Some(30),
        entity_type: BlockedUrlEntityType::Page,
        request_type: BlockedUrlRequestType::FullRemoval,
    };
    client.add_blocked_url(site_url, &blocked_url).await?;

    // Add page preview block
    println!("Adding page preview block...");
    client
        .add_page_preview_block(
            site_url,
            "https://example.com/private",
            BlockReason::NoPreview,
        )
        .await?;

    // Add deep link block
    println!("Adding deep link block...");
    client
        .add_deep_link_block(
            site_url,
            "en-US",
            "https://www.bing.com/search?q=example",
            "https://example.com/product/123",
        )
        .await?;

    // Set geographic targeting
    println!("Setting country/region targeting...");
    let geo_settings = CountryRegionSettings {
        date: Utc::now(),
        two_letter_iso_country_code: "US".to_string(),
        r#type: CountryRegionSettingsType::Domain,
        url: site_url.to_string(),
    };
    client
        .add_country_region_settings(site_url, &geo_settings)
        .await?;

    // Add connected pages (pages that link to your site)
    println!("Adding connected pages...");
    client
        .add_connected_page(site_url, "https://partner-site.com/links-to-us")
        .await?;

    // Get connected pages
    println!("Getting connected pages...");
    let connected = client.get_connected_pages(site_url).await?;
    for page in &connected {
        println!(
            "  Connected page: {} (verified: {})",
            page.url, page.is_verified
        );
    }

    // Request crawling of specific URLs
    println!("Requesting crawl of specific URLs...");
    client
        .fetch_url(site_url, "https://example.com/new-content")
        .await?;

    println!("URL management operations completed!");
    Ok(())
}
