use crate::dto::*;
use crate::error::{map_status_error, try_parse_api_error, Result, WebmasterApiError};
use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde_json::json;
use tracing::instrument;

/// Bing Webmaster API client for interacting with Bing Webmaster Tools
///
/// This client provides access to all methods from the Bing Webmaster API,
/// allowing you to manage your websites, submit URLs, track crawl issues, and more.
#[derive(Debug, Clone)]
pub struct BingWebmasterClient {
    client: ClientWithMiddleware,
    base_url: String,
    api_key: String,
}

impl BingWebmasterClient {
    /// Helper method to handle API responses with proper error handling
    async fn handle_response<T: for<'de> serde::Deserialize<'de>>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let response_text = response.text().await?;

            println!("{}", &response_text);

            // Try to deserialize as wrapped response
            match serde_json::from_str::<ResponseWrapper<T>>(&response_text) {
                Ok(wrapper) => Ok(wrapper.d),
                Err(json_err) => {
                    // Try to deserialize directly
                    match serde_json::from_str::<T>(&response_text) {
                        Ok(data) => Ok(data),
                        Err(_) => {
                            // Check if response contains an API error
                            if let Some((error_code, message)) = try_parse_api_error(&response_text)
                            {
                                Err(WebmasterApiError::api_error(
                                    error_code,
                                    message,
                                    Some(status.as_u16()),
                                ))
                            } else {
                                Err(WebmasterApiError::invalid_response(format!(
                                    "Failed to parse response: {}",
                                    json_err
                                )))
                            }
                        }
                    }
                }
            }
        } else {
            let response_text = response.text().await.unwrap_or_default();

            // Check if response contains an API error
            if let Some((error_code, message)) = try_parse_api_error(&response_text) {
                Err(WebmasterApiError::api_error(
                    error_code,
                    message,
                    Some(status.as_u16()),
                ))
            } else {
                Err(map_status_error(status, response_text))
            }
        }
    }

    /// Helper method to handle void responses (methods that return void in .NET)
    async fn handle_void_response(&self, response: Response) -> Result<()> {
        let status = response.status();

        if status.is_success() {
            Ok(())
        } else {
            let response_text = response.text().await.unwrap_or_default();

            // Check if response contains an API error
            if let Some((error_code, message)) = try_parse_api_error(&response_text) {
                Err(WebmasterApiError::api_error(
                    error_code,
                    message,
                    Some(status.as_u16()),
                ))
            } else {
                Err(map_status_error(status, response_text))
            }
        }
    }

    pub fn new(api_key: String) -> Self {
        let client = ClientBuilder::new(Client::new()).build();
        Self {
            client,
            base_url: "https://ssl.bing.com/webmaster/api.svc".to_string(),
            api_key,
        }
    }

    /// Create a new WebmasterApiClient with default reqwest client
    ///
    /// # Arguments
    /// * `api_key` - Your Bing Webmaster Tools API key
    /// * `base_url` - Optional custom base URL (defaults to official Bing API endpoint)
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        let client = ClientBuilder::new(Client::new()).build();
        Self {
            client,
            base_url,
            api_key,
        }
    }

    /// Create a new WebmasterApiClient with custom middleware
    ///
    /// # Arguments
    /// * `api_key` - Your Bing Webmaster Tools API key
    /// * `base_url` - Optional custom base URL (defaults to official Bing API endpoint)
    /// * `client_builder` - ClientBuilder with configured middleware
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bing_webmaster_api::BingWebmasterClient;
    /// use reqwest::Client;
    /// use reqwest_middleware::ClientBuilder;
    ///
    /// let client_builder = ClientBuilder::new(Client::new());
    /// let api_client = BingWebmasterClient::with_middleware(
    ///     "your-api-key".to_string(),
    ///     None,
    ///     client_builder
    /// );
    /// ```
    pub fn with_middleware(
        api_key: String,
        base_url: Option<String>,
        client_builder: ClientBuilder,
    ) -> Self {
        Self {
            client: client_builder.build(),
            base_url: base_url
                .unwrap_or_else(|| "https://ssl.bing.com/webmaster/api.svc".to_string()),
            api_key,
        }
    }

    // Write methods (return void in .NET API)

    #[instrument(skip(self))]
    pub async fn add_blocked_url(&self, site_url: &str, blocked_url: &BlockedUrl) -> Result<()> {
        let url = format!(
            "{}/json/AddBlockedUrl?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "blockedUrl": blocked_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn add_connected_page(&self, site_url: &str, master_url: &str) -> Result<()> {
        let url = format!(
            "{}/json/AddConnectedPage?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "masterUrl": master_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn add_country_region_settings(
        &self,
        site_url: &str,
        settings: &CountryRegionSettings,
    ) -> Result<()> {
        let url = format!(
            "{}/json/AddCountryRegionSettings?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "settings": settings
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn add_deep_link_block(
        &self,
        site_url: &str,
        market: &str,
        search_url: &str,
        deep_link_url: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/json/AddDeepLinkBlock?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "market": market,
            "searchUrl": search_url,
            "deepLinkUrl": deep_link_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn add_query_parameter(&self, site_url: &str, query_parameter: &str) -> Result<()> {
        let url = format!(
            "{}/json/AddQueryParameter?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "queryParameter": query_parameter
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn add_site(&self, site_url: &str) -> Result<()> {
        let url = format!("{}/json/AddSite?apikey={}", self.base_url, self.api_key);
        let body = json!({
            "siteUrl": site_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn add_site_roles(
        &self,
        site_url: &str,
        delegated_url: &str,
        user_email: &str,
        authentication_code: &str,
        is_administrator: bool,
        is_read_only: bool,
    ) -> Result<()> {
        let url = format!(
            "{}/json/AddSiteRoles?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "delegatedUrl": delegated_url,
            "userEmail": user_email,
            "authenticationCode": authentication_code,
            "isAdministrator": is_administrator,
            "isReadOnly": is_read_only
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn enable_disable_query_parameter(
        &self,
        site_url: &str,
        parameter: &str,
        enabled: bool,
    ) -> Result<()> {
        let url = format!(
            "{}/json/EnableDisableQueryParameter?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "queryParameter": parameter,
            "isEnabled": enabled
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn fetch_url(&self, site_url: &str, url: &str) -> Result<()> {
        let api_url = format!("{}/json/FetchUrl?apikey={}", self.base_url, self.api_key);
        let body = json!({
            "siteUrl": site_url,
            "url": url
        });

        let response = self
            .client
            .post(&api_url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    /// Submit content
    ///
    /// ### Parameters
    /// * siteUrl - Site url E.g.: http://example.com
    /// * url - Url to submit E.g.: http://example.com/url1.html
    /// * httpMessage - HTTP status line, such as HTTP/1.1 200 OK, HTTP Headers, an empty line and optional HTTP message body data equivalent of the response based64 encoded if bingbot was fetching this URL. The request/status line and headers must all end with <CR> <LF> (that is, a carriage return followed by a line feed). The empty line must consist of only <CR> <LF> and no other whitespace.
    /// * structuredData - Structured Data (typically JSON-LD ) provided based64 encoded typically used for submitting to bing structured Data for non-HTML content types as images, PDF files.Empty if no structured data provided.
    /// * dynamicServing - {none = 0, PC-laptop = 1, mobile = 2, AMP = 3, tablet = 4, non-visual browser = 5}. Set this field to a value greater than 0 only if your web site dynamically serves different content based on customer devices visiting your web site
    #[instrument(skip(self))]
    pub async fn submit_content(
        &self,
        site_url: &str,
        url: &str,
        http_message: &str,
        structured_data: &str,
        dynamic_serving: i32,
    ) -> Result<()> {
        let api_url = format!(
            "{}/json/SubmitContent?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "url": url,
            "httpMessage": http_message,
            "structuredData": structured_data,
            "dynamicServing": dynamic_serving
        });

        let response = self
            .client
            .post(&api_url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn submit_url(&self, site_url: &str, url: &str) -> Result<()> {
        let api_url = format!("{}/json/SubmitUrl?apikey={}", self.base_url, self.api_key);
        let body = json!({
            "siteUrl": site_url,
            "url": url
        });

        let response = self
            .client
            .post(&api_url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn submit_url_batch(&self, site_url: &str, url_list: &[String]) -> Result<()> {
        let api_url = format!(
            "{}/json/SubmitUrlBatch?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "urlList": url_list
        });

        let response = self
            .client
            .post(&api_url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    // Read methods (return data)

    #[instrument(skip(self))]
    pub async fn get_crawl_issues(&self, site_url: &str) -> Result<Vec<UrlWithCrawlIssues>> {
        let url = format!(
            "{}/json/GetCrawlIssues?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_page_stats(&self, site_url: &str) -> Result<Vec<QueryStats>> {
        let url = format!(
            "{}/json/GetPageStats?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_connected_pages(&self, site_url: &str) -> Result<Vec<Site>> {
        let url = format!(
            "{}/json/GetConnectedPages?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_query_parameters(&self, site_url: &str) -> Result<Vec<QueryParameter>> {
        let url = format!(
            "{}/json/GetQueryParameters?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_crawl_stats(&self, site_url: &str) -> Result<Vec<CrawlStats>> {
        let url = format!(
            "{}/json/GetCrawlStats?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn verify_site(&self, site_url: &str) -> Result<bool> {
        let url = format!("{}/json/VerifySite?apikey={}", self.base_url, self.api_key);
        let body = json!({
            "siteUrl": site_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_content_submission_quota(
        &self,
        site_url: &str,
    ) -> Result<ContentSubmissionQuota> {
        let url = format!(
            "{}/json/GetContentSubmissionQuota?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_blocked_urls(&self, site_url: &str) -> Result<Vec<BlockedUrl>> {
        let url = format!(
            "{}/json/GetBlockedUrls?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_children_url_info(
        &self,
        site_url: &str,
        url: &str,
        page: u16,
        filter_properties: &FilterProperties,
    ) -> Result<Vec<UrlInfo>> {
        let api_url = format!(
            "{}/json/GetChildrenUrlInfo?apikey={}",
            self.base_url, self.api_key
        );

        let body = serde_json::json!({
            "siteUrl": site_url,
            "url": url,
            "page": page,
            "filterProperties": filter_properties
        });

        let response = self
            .client
            .post(&api_url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_children_url_traffic_info(
        &self,
        site_url: &str,
        url: &str,
        page: u16,
    ) -> Result<Vec<UrlTrafficInfo>> {
        let api_url = format!(
            "{}/json/GetChildrenUrlTrafficInfo?apikey={}&siteUrl={}&url={}&page={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(url),
            page
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_country_region_settings(
        &self,
        site_url: &str,
    ) -> Result<Vec<CountryRegionSettings>> {
        let url = format!(
            "{}/json/GetCountryRegionSettings?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_crawl_settings(&self, site_url: &str) -> Result<CrawlSettings> {
        let url = format!(
            "{}/json/GetCrawlSettings?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_deep_link(&self, site_url: &str, url: &str) -> Result<DeepLink> {
        let api_url = format!(
            "{}/json/GetDeepLink?apikey={}&siteUrl={}&url={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(url)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_deep_link_algo_urls(&self, site_url: &str) -> Result<Vec<DeepLinkAlgoUrl>> {
        let url = format!(
            "{}/json/GetDeepLinkAlgoUrls?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_deep_link_blocks(&self, site_url: &str) -> Result<Vec<DeepLinkBlock>> {
        let url = format!(
            "{}/json/GetDeepLinkBlocks?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_feed_details(&self, site_url: &str, feed_url: &str) -> Result<Vec<Feed>> {
        let api_url = format!(
            "{}/json/GetFeedDetails?apikey={}&siteUrl={}&feedUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(feed_url)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_feeds(&self, site_url: &str) -> Result<Vec<Feed>> {
        let url = format!(
            "{}/json/GetFeeds?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_fetched_url_details(
        &self,
        site_url: &str,
        url: &str,
    ) -> Result<FetchedUrlDetails> {
        let api_url = format!(
            "{}/json/GetFetchedUrlDetails?apikey={}&siteUrl={}&url={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(url)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_fetched_urls(&self, site_url: &str) -> Result<Vec<FetchedUrl>> {
        let url = format!(
            "{}/json/GetFetchedUrls?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_keyword(
        &self,
        query: &str,
        country: &str,
        language: &str,
        start_date: chrono::DateTime<chrono::Utc>,
        end_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Keyword>> {
        let api_url = format!(
            "{}/json/GetKeyword?apikey={}&q={}&country={}&language={}&startDate={}&endDate={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(query),
            urlencoding::encode(country),
            urlencoding::encode(language),
            start_date.format("%Y-%m-%dT%H:%M:%S"),
            end_date.format("%Y-%m-%dT%H:%M:%S")
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_keyword_stats(
        &self,
        query: &str,
        country: &str,
        language: &str,
    ) -> Result<Vec<KeywordStats>> {
        let api_url = format!(
            "{}/json/GetKeywordStats?apikey={}&q={}&country={}&language={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(query),
            urlencoding::encode(country),
            urlencoding::encode(language)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_link_counts(&self, site_url: &str, page: i16) -> Result<LinkCounts> {
        let url = format!(
            "{}/json/GetLinkCounts?apikey={}&siteUrl={}&page={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            page
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_page_query_stats(
        &self,
        site_url: &str,
        page_url: &str,
    ) -> Result<Vec<QueryStats>> {
        let api_url = format!(
            "{}/json/GetPageQueryStats?apikey={}&siteUrl={}&page={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(page_url)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_query_page_detail_stats(
        &self,
        site_url: &str,
        query: &str,
        page_url: &str,
    ) -> Result<Vec<DetailedQueryStats>> {
        let api_url = format!(
            "{}/json/GetQueryPageDetailStats?apikey={}&siteUrl={}&query={}&page={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(query),
            urlencoding::encode(page_url)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_query_page_stats(
        &self,
        site_url: &str,
        query: &str,
    ) -> Result<Vec<QueryStats>> {
        let api_url = format!(
            "{}/json/GetQueryPageStats?apikey={}&siteUrl={}&query={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(query)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_query_stats(&self, site_url: &str) -> Result<Vec<QueryStats>> {
        let url = format!(
            "{}/json/GetQueryStats?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_query_traffic_stats(
        &self,
        site_url: &str,
        query: &str,
    ) -> Result<Vec<RankAndTrafficStats>> {
        let api_url = format!(
            "{}/json/GetQueryTrafficStats?apikey={}&siteUrl={}&query={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(query)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_rank_and_traffic_stats(
        &self,
        site_url: &str,
    ) -> Result<Vec<RankAndTrafficStats>> {
        let url = format!(
            "{}/json/GetRankAndTrafficStats?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_site_moves(&self, site_url: &str) -> Result<Vec<SiteMove>> {
        let url = format!(
            "{}/json/GetSiteMoves?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_site_roles(
        &self,
        site_url: &str,
        include_all_subdomains: bool,
    ) -> Result<Vec<SiteRoles>> {
        let url = format!(
            "{}/json/GetSiteRoles?apikey={}&siteUrl={}&includeAllSubdomains={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            include_all_subdomains
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_url_info(&self, site_url: &str, url: &str) -> Result<UrlInfo> {
        let api_url = format!(
            "{}/json/GetUrlInfo?apikey={}&siteUrl={}&url={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(url)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_url_links(
        &self,
        site_url: &str,
        link: &str,
        page: i16,
    ) -> Result<LinkDetails> {
        let api_url = format!(
            "{}/json/GetUrlLinks?apikey={}&siteUrl={}&link={}&page={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(link),
            page
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_url_submission_quota(&self, site_url: &str) -> Result<UrlSubmissionQuota> {
        let url = format!(
            "{}/json/GetUrlSubmissionQuota?apikey={}&siteUrl={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url)
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_url_traffic_info(&self, site_url: &str, url: &str) -> Result<UrlTrafficInfo> {
        let api_url = format!(
            "{}/json/GetUrlTrafficInfo?apikey={}&siteUrl={}&url={}",
            self.base_url,
            self.api_key,
            urlencoding::encode(site_url),
            urlencoding::encode(url)
        );

        let response = self.client.get(&api_url).send().await?;

        self.handle_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn get_user_sites(&self) -> Result<Vec<Site>> {
        let url = format!(
            "{}/json/GetUserSites?apikey={}",
            self.base_url, self.api_key
        );

        let response = self.client.get(&url).send().await?;

        self.handle_response(response).await
    }

    // Remove/Delete methods

    #[instrument(skip(self))]
    pub async fn remove_blocked_url(&self, site_url: &str, blocked_url: &BlockedUrl) -> Result<()> {
        let url = format!(
            "{}/json/RemoveBlockedUrl?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "blockedUrl": blocked_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn remove_country_region_settings(
        &self,
        site_url: &str,
        settings: &CountryRegionSettings,
    ) -> Result<()> {
        let url = format!(
            "{}/json/RemoveCountryRegionSettings?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "settings": settings
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn remove_deep_link_block(
        &self,
        site_url: &str,
        market: &str,
        search_url: &str,
        deep_link_url: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/json/RemoveDeepLinkBlock?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "market": market,
            "searchUrl": search_url,
            "deepLinkUrl": deep_link_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn remove_feed(&self, site_url: &str, feed_url: &str) -> Result<()> {
        let url = format!("{}/json/RemoveFeed?apikey={}", self.base_url, self.api_key);
        let body = json!({
            "siteUrl": site_url,
            "feedUrl": feed_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn remove_query_parameter(
        &self,
        site_url: &str,
        query_parameter: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/json/RemoveQueryParameter?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "queryParameter": query_parameter
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn remove_site(&self, site_url: &str) -> Result<()> {
        let url = format!("{}/json/RemoveSite?apikey={}", self.base_url, self.api_key);
        let body = json!({
            "siteUrl": site_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn remove_site_role(&self, site_url: &str, site_roles: &SiteRoles) -> Result<()> {
        let url = format!(
            "{}/json/RemoveSiteRole?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "siteRoles": site_roles
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    // Save/Submit methods

    #[instrument(skip(self))]
    pub async fn save_crawl_settings(
        &self,
        site_url: &str,
        crawl_settings: &CrawlSettings,
    ) -> Result<()> {
        let url = format!(
            "{}/json/SaveCrawlSettings?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "crawlSettings": crawl_settings
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn submit_feed(&self, site_url: &str, feed_url: &str) -> Result<()> {
        let url = format!("{}/json/SubmitFeed?apikey={}", self.base_url, self.api_key);
        let body = json!({
            "siteUrl": site_url,
            "feedUrl": feed_url
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn submit_site_move(
        &self,
        site_url: &str,
        site_move_settings: &SiteMoveSettings,
    ) -> Result<()> {
        let url = format!(
            "{}/json/SubmitSiteMove?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "siteMoveSettings": site_move_settings
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }

    #[instrument(skip(self))]
    pub async fn update_deep_link(
        &self,
        site_url: &str,
        market: &str,
        search_url: &str,
        deep_link_weight: &DeepLinkWeight,
    ) -> Result<()> {
        let url = format!(
            "{}/json/UpdateDeepLink?apikey={}",
            self.base_url, self.api_key
        );
        let body = json!({
            "siteUrl": site_url,
            "market": market,
            "searchUrl": search_url,
            "deepLinkWeight": deep_link_weight
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .body(serde_json::to_string(&body)?)
            .send()
            .await?;

        self.handle_void_response(response).await
    }
}
