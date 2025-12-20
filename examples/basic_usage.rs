use bing_webmaster_api::{BingWebmasterClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the client with your API key
    let client = BingWebmasterClient::new(
        std::env::var("BING_WEBMASTER_API_KEY")
            .expect("BING_WEBMASTER_API_KEY environment variable must be set"),
    );

    let site_url = "https://example.com";

    // Add a site to your account
    println!("Adding site: {}", site_url);
    client.add_site(site_url).await?;

    // Verify the site
    println!("Verifying site...");
    let verified = client.verify_site(site_url).await?;
    println!("Site verified: {}", verified);

    // Submit a single URL
    let page_url = "https://example.com/new-page";
    println!("Submitting URL: {}", page_url);
    client.submit_url(site_url, page_url).await?;

    // Submit multiple URLs
    let urls = vec![
        "https://example.com/page1".to_string(),
        "https://example.com/page2".to_string(),
        "https://example.com/page3".to_string(),
    ];
    println!("Submitting {} URLs...", urls.len());
    client.submit_url_batch(site_url, &urls).await?;

    // Check submission quota
    println!("Checking submission quota...");
    let quota = client.get_content_submission_quota(site_url).await?;
    println!(
        "Daily quota: {}, Monthly quota: {}",
        quota.daily_quota, quota.monthly_quota
    );

    // Get crawl issues
    println!("Getting crawl issues...");
    let issues = client.get_crawl_issues(site_url).await?;
    println!("Found {} URLs with crawl issues", issues.len());

    for issue_url in issues.iter().take(5) {
        println!("  URL: {}", issue_url.url);
        println!(
            "    HTTP Code: {}, Issues: {:?}, In Links: {}",
            issue_url.http_code, issue_url.issues, issue_url.in_links
        );
    }

    // Get page statistics
    println!("Getting page statistics...");
    let stats = client.get_page_stats(site_url).await?;
    println!("Found stats for {} queries", stats.len());

    for stat in stats.iter().take(5) {
        println!(
            "  Query: '{}' - {} impressions, {} clicks (CTR: {:.2}%)",
            stat.query,
            stat.impressions,
            stat.clicks,
            if stat.impressions > 0 {
                (stat.clicks as f64 / stat.impressions as f64) * 100.0
            } else {
                0.0
            }
        );
    }

    // Get crawl statistics
    println!("Getting crawl statistics...");
    let crawl_stats = client.get_crawl_stats(site_url).await?;
    println!("Found crawl stats for {} time periods", crawl_stats.len());

    for stat in crawl_stats.iter().take(5) {
        println!(
            "  Date: {} - {} pages crawled, {} errors, {} warnings",
            stat.date.format("%Y-%m-%d"),
            stat.crawled_pages,
            stat.crawl_errors,
            stat.all_other_codes
        );
    }

    println!("Done!");
    Ok(())
}
