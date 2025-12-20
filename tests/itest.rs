use bing_webmaster_api::{
    BingWebmasterClient, CountryRegionSettings, CountryRegionSettingsType, CrawlSettings,
};
use chrono::Utc;
use rand::distr::{Alphanumeric, SampleString};
use std::fs::File;
use std::io::Read;

async fn new_client() -> anyhow::Result<BingWebmasterClient> {
    let mut str = String::new();
    File::open("tests/token")?.read_to_string(&mut str)?;

    Ok(BingWebmasterClient::new(str))
}

fn generate_random_host() -> String {
    let s: String = Alphanumeric.sample_string(&mut rand::rng(), 8);
    format!("example-{s}.com")
}

#[tokio::test]
#[ignore]
async fn get_sites() -> anyhow::Result<()> {
    let client = new_client().await?;

    let sites = client.get_user_sites().await?;

    dbg!(sites);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn add_and_manage_site() -> anyhow::Result<()> {
    let client = new_client().await?;

    let example = format!("https://{}/", generate_random_host()).to_lowercase();

    client.add_site(&example).await?;

    let sites = client.get_user_sites().await?;
    dbg!(&example);
    dbg!(&sites);
    let site = sites.iter().find(|s| s.url == example).unwrap();

    let verify = client.verify_site(&example).await?;
    assert!(!verify);

    let roles = client.get_site_roles("https://seomeow.com/", true).await?;
    dbg!(roles);

    // client
    //     .add_site_roles(
    //         &example,
    //         &example,
    //         "test@example.com",
    //         &site.authentication_code,
    //         false,
    //         true,
    //     )
    //     .await?;
    //
    // let roles = client.get_site_roles(&example, true).await?;
    // dbg!(&roles);
    //
    // let role = roles.into_iter().find(|s| s.email == "test@example.com").unwrap();
    //
    // client.remove_site_role(&example, &role).await?;

    // client.add_connected_page(&example, "https://example.com/").await?;

    // let pages = client.get_connected_pages(&example).await?;

    // dbg!(pages);

    let settings = client.get_crawl_settings(&example).await?;

    dbg!(&settings);

    client.remove_site(&example).await?;

    let sites = client.get_user_sites().await?;
    assert!(!sites.iter().any(|s| s.url == example));

    Ok(())
}

#[tokio::test]
#[ignore]
async fn manage_site() -> anyhow::Result<()> {
    let client = new_client().await?;

    let site = client
        .get_user_sites()
        .await?
        .into_iter()
        .find(|s| s.is_verified)
        .unwrap();

    client
        .add_connected_page(&site.url, "https://example.com/")
        .await?;

    let pages = client.get_connected_pages(&site.url).await?;

    dbg!(pages);

    let settings = client.get_crawl_settings(&site.url).await?;

    dbg!(&settings);

    client
        .save_crawl_settings(
            &site.url,
            &CrawlSettings {
                crawl_boost_available: true,
                crawl_boost_enabled: true,
                crawl_rate: settings.crawl_rate,
            },
        )
        .await?;

    let settings = CountryRegionSettings {
        date: Utc::now().date_naive(),
        two_letter_iso_country_code: "ru".to_string(),
        r#type: CountryRegionSettingsType::Page,
        url: "https://seomeow.com".to_string(),
    };
    client
        .add_country_region_settings(&site.url, &settings)
        .await?;

    let country = client.get_country_region_settings(&site.url).await?;

    dbg!(country);

    client
        .remove_country_region_settings(&site.url, &settings)
        .await?;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn submits() -> anyhow::Result<()> {
    let client = new_client().await?;

    let site = client
        .get_user_sites()
        .await?
        .into_iter()
        .find(|s| s.is_verified)
        .unwrap();

    let quota = client.get_content_submission_quota(&site.url).await?;

    dbg!(quota);

    let quota = client.get_url_submission_quota(&site.url).await?;

    dbg!(quota);

    client.submit_url(&site.url, &site.url).await?;

    client
        .submit_url_batch(&site.url, &vec![site.url.to_string(), site.url.to_string()])
        .await?;

    client.submit_content(&site.url, &format!("{}/broken", &site.url), "SFRUUC8xLjEgMjAwIE9LDQpEYXRlOiBTdW4sIDEwIE9jdCAyMDE3IDIzOjI2OjA3IEdNVA0KQWNjZXB0LVJhbmdlczogYnl0ZXMNCkNvbnRlbnQtTGVuZ3RoOiAxMTMNCkNvbm5lY3Rpb246IGNsb3NlDQpDb250ZW50LVR5cGU6IHRleHQvaHRtbA0KDQo8IURPQ1RZUEUgaHRtbD4NCjxodG1sPg0KPGhlYWQ+DQo8dGl0bGU+VGVzdCBQYWdlPC90aXRsZT4NCjwvaGVhZD4NCjxib2R5Pg0KPHA+SGVsbG8gd29ybGQhPC9wPg0KPC9ib2R5Pg0KPC9odG1sPg==", "", 0).await?;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn feeds() -> anyhow::Result<()> {
    let client = new_client().await?;

    let site = client
        .get_user_sites()
        .await?
        .into_iter()
        .find(|s| s.is_verified)
        .unwrap();

    let feed = format!("{}/sitemap-broken.xml", &site.url);

    client.submit_feed(&site.url, &feed).await?;

    let feeds = client.get_feeds(&site.url).await?;
    dbg!(&feeds);
    assert!(feeds.iter().any(|f| f.url == feed));

    client.remove_feed(&site.url, &feed).await?;

    let feeds = client.get_feeds(&site.url).await?;
    dbg!(&feeds);
    assert!(!feeds.iter().any(|f| f.url == feed));

    let index = feeds
        .into_iter()
        .find(|s| s.r#type == "Sitemap Index")
        .unwrap();

    let feeds = client.get_feed_details(&site.url, &index.url).await?;
    dbg!(&feeds);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn fetched_urls() -> anyhow::Result<()> {
    let client = new_client().await?;

    let site = client
        .get_user_sites()
        .await?
        .into_iter()
        .find(|s| s.is_verified)
        .unwrap();

    client.fetch_url(&site.url, &site.url).await?;

    let fetched = client.get_fetched_urls(&site.url).await?;

    dbg!(&fetched);

    let details = client
        .get_fetched_url_details(&site.url, &fetched.first().unwrap().url)
        .await?;

    dbg!(&details);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn stats() -> anyhow::Result<()> {
    let client = new_client().await?;

    let site = client
        .get_user_sites()
        .await?
        .into_iter()
        .find(|s| s.is_verified)
        .unwrap();

    let stats = client.get_query_stats(&site.url).await?;

    dbg!(stats);

    let stats = client.get_page_stats(&site.url).await?;

    dbg!(stats);

    let stats = client.get_rank_and_traffic_stats(&site.url).await?;

    dbg!(stats);

    let stats = client.get_crawl_stats(&site.url).await?;

    dbg!(stats);

    let issues = client.get_crawl_issues(&site.url).await?;

    dbg!(&issues);

    let info = client.get_url_info(&site.url, &site.url).await?;

    dbg!(&info);

    let traffic = client.get_url_traffic_info(&site.url, &site.url).await?;

    dbg!(&traffic);

    let params = client.get_query_parameters(&site.url).await?;

    dbg!(&params);

    client.add_query_parameter(&site.url, "test").await?;

    let params = client.get_query_parameters(&site.url).await?;

    dbg!(&params);
    assert!(params.iter().any(|s| s.parameter == "test"));

    client
        .enable_disable_query_parameter(&site.url, "test", false)
        .await?;
    client
        .enable_disable_query_parameter(&site.url, "test", true)
        .await?;

    client.remove_query_parameter(&site.url, "test").await?;

    let params = client.get_query_parameters(&site.url).await?;

    dbg!(&params);
    assert!(!params.iter().any(|s| s.parameter == "test"));

    let stats = client.get_query_traffic_stats(&site.url, "seo").await?;

    dbg!(stats);

    let stats = client.get_query_page_stats(&site.url, &site.url).await?;

    dbg!(&stats);

    let stats = client
        .get_query_page_detail_stats(&site.url, "seo", &site.url)
        .await?;
    dbg!(&stats);

    let stats = client.get_page_query_stats(&site.url, &site.url).await?;
    dbg!(&stats);

    let links = client.get_link_counts(&site.url, 0).await?;

    dbg!(links);

    let links = client.get_url_links(&site.url, &site.url, 0).await?;
    dbg!(links);

    let info = client
        .get_children_url_traffic_info(&site.url, &site.url, 0)
        .await?;

    dbg!(&info);

    use bing_webmaster_api::{
        CrawlDateFilter, DiscoveredDateFilter, DocFlagsFilters, FilterProperties, HttpCodeFilters,
    };

    let filter = FilterProperties {
        crawl_date_filter: CrawlDateFilter::Any,
        discovered_date_filter: DiscoveredDateFilter::Any,
        doc_flags_filters: DocFlagsFilters::Any,
        http_code_filters: HttpCodeFilters::Any,
    };

    let info = client
        .get_children_url_info(&site.url, &site.url, 0, &filter)
        .await?;

    dbg!(&info);

    // let stats = client.get_keyword_stats("seo", "ru", "RU_ru").await?;
    //
    // dbg!(stats);

    Ok(())
}
