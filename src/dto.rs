//! Data Transfer Objects (DTOs) for Bing Webmaster API
//!
//! This module contains all data structures used for communication with the Bing Webmaster API.
//! All structures mirror the .NET API definitions from `Microsoft.Bing.Webmaster.Api.Interfaces`.
//!
//! # Field Naming
//!
//! All fields use `#[serde(rename = "...")]` to match the PascalCase naming convention
//! used by the .NET API, while providing idiomatic snake_case Rust field names.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// .NET DateTime serialization format used by Bing API
///
/// The Bing API uses .NET's JSON date format: `/Date(timestamp-offset)/`
/// where timestamp is milliseconds since Unix epoch.
///
/// # Format
/// `/Date(1316156400000-0700)/`
/// - `1316156400000` - milliseconds since Unix epoch
/// - `-0700` - timezone offset (optional)
mod dotnet_date_format {
    use chrono::{DateTime, NaiveDate, TimeZone, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp_ms = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();
        let formatted = format!("/Date({})/", timestamp_ms);
        formatted.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Handle special case: dates starting with /Date(- are considered null
        if s.starts_with("/Date(-") {
            return Err(serde::de::Error::custom("Null date value"));
        }

        // Handle .NET date format: "/Date(1316156400000-0700)/"
        if s.starts_with("/Date(") && s.ends_with(")/") {
            let inner = &s[6..s.len() - 2]; // Remove "/Date(" and ")/"

            // Find last - or + for timezone offset
            let (hyph_pos, is_negative) = if let Some(pos) = inner.rfind('-') {
                (Some(pos), true)
            } else if let Some(pos) = inner.rfind('+') {
                (Some(pos), false)
            } else {
                (None, true)
            };

            if let Some(hyph) = hyph_pos {
                // Parse timestamp before timezone offset
                let timestamp_str = &inner[..hyph];
                let mut timestamp_ms = timestamp_str
                    .parse::<f64>()
                    .map_err(|_| serde::de::Error::custom("Failed to parse timestamp"))?;

                // Parse timezone offset hours (2 digits)
                if hyph + 3 <= inner.len() {
                    let hours_str = &inner[hyph + 1..hyph + 3];
                    let hours = hours_str
                        .parse::<f64>()
                        .map_err(|_| serde::de::Error::custom("Failed to parse hours"))?;

                    // Parse timezone offset minutes (2 digits)
                    let mins = if hyph + 5 <= inner.len() {
                        let mins_str = &inner[hyph + 3..hyph + 5];
                        mins_str.parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    };

                    // Apply timezone offset to convert to UTC
                    let offset_ms = (hours * 60.0 * 60.0 * 1000.0) + (mins * 60.0 * 1000.0);
                    if is_negative {
                        timestamp_ms -= offset_ms;
                    } else {
                        timestamp_ms += offset_ms;
                    }
                }

                // Create DateTime from adjusted timestamp
                match Utc.timestamp_millis_opt(timestamp_ms as i64) {
                    chrono::LocalResult::Single(dt) => Ok(dt.date_naive()),
                    _ => Err(serde::de::Error::custom("Invalid timestamp")),
                }
            } else {
                // No timezone offset - parse as timestamp and use date only (no time component)
                let timestamp_ms = inner
                    .parse::<i64>()
                    .map_err(|_| serde::de::Error::custom("Failed to parse timestamp"))?;

                match Utc.timestamp_millis_opt(timestamp_ms) {
                    chrono::LocalResult::Single(dt) => {
                        // Return date only (time set to 00:00:00)
                        let date = dt.date_naive();
                        Ok(date)
                    }
                    _ => Err(serde::de::Error::custom("Invalid timestamp")),
                }
            }
        } else {
            // Fallback to standard ISO format
            s.parse::<DateTime<Utc>>()
                .map(|s| s.date_naive())
                .map_err(serde::de::Error::custom)
        }
    }
}

mod dotnet_date_format_opt {
    use chrono::{DateTime, NaiveDate, TimeZone, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            None => "/Date(-0)/".serialize(serializer),
            Some(date) => {
                let timestamp_ms = date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();
                let formatted = format!("/Date({})/", timestamp_ms);
                formatted.serialize(serializer)
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Handle special case: dates starting with /Date(- are considered null
        if s.starts_with("/Date(-") {
            return Ok(None);
        }

        // Handle .NET date format: "/Date(1316156400000-0700)/"
        if s.starts_with("/Date(") && s.ends_with(")/") {
            let inner = &s[6..s.len() - 2]; // Remove "/Date(" and ")/"

            // Find last - or + for timezone offset
            let (hyph_pos, is_negative) = if let Some(pos) = inner.rfind('-') {
                (Some(pos), true)
            } else if let Some(pos) = inner.rfind('+') {
                (Some(pos), false)
            } else {
                (None, true)
            };

            if let Some(hyph) = hyph_pos {
                // Parse timestamp before timezone offset
                let timestamp_str = &inner[..hyph];
                let mut timestamp_ms = timestamp_str
                    .parse::<f64>()
                    .map_err(|_| serde::de::Error::custom("Failed to parse timestamp"))?;

                // Parse timezone offset hours (2 digits)
                if hyph + 3 <= inner.len() {
                    let hours_str = &inner[hyph + 1..hyph + 3];
                    let hours = hours_str
                        .parse::<f64>()
                        .map_err(|_| serde::de::Error::custom("Failed to parse hours"))?;

                    // Parse timezone offset minutes (2 digits)
                    let mins = if hyph + 5 <= inner.len() {
                        let mins_str = &inner[hyph + 3..hyph + 5];
                        mins_str.parse::<f64>().unwrap_or(0.0)
                    } else {
                        0.0
                    };

                    // Apply timezone offset to convert to UTC
                    let offset_ms = (hours * 60.0 * 60.0 * 1000.0) + (mins * 60.0 * 1000.0);
                    if is_negative {
                        timestamp_ms -= offset_ms;
                    } else {
                        timestamp_ms += offset_ms;
                    }
                }

                // Create DateTime from adjusted timestamp
                match Utc.timestamp_millis_opt(timestamp_ms as i64) {
                    chrono::LocalResult::Single(dt) => Ok(Some(dt.date_naive())),
                    _ => Err(serde::de::Error::custom("Invalid timestamp")),
                }
            } else {
                // No timezone offset - parse as timestamp and use date only (no time component)
                let timestamp_ms = inner
                    .parse::<i64>()
                    .map_err(|_| serde::de::Error::custom("Failed to parse timestamp"))?;

                match Utc.timestamp_millis_opt(timestamp_ms) {
                    chrono::LocalResult::Single(dt) => {
                        // Return date only (time set to 00:00:00)
                        let date = dt.date_naive();
                        Ok(Some(date))
                    }
                    _ => Err(serde::de::Error::custom("Invalid timestamp")),
                }
            }
        } else {
            // Fallback to standard ISO format
            s.parse::<DateTime<Utc>>()
                .map(|s| Some(s.date_naive()))
                .map_err(serde::de::Error::custom)
        }
    }
}

/// Response wrapper for Bing Webmaster API JSON responses
///
/// All JSON responses from the Bing API are wrapped in a `{"d": data}` structure,
/// following the .NET WCF JSON serialization format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseWrapper<T> {
    /// The wrapped response data
    pub d: T,
}

/// Represents a URL that has been blocked from Bing's search index
///
/// Used to request temporary removal of content from Bing search results.
/// This can be for a single page or an entire directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedUrl {
    /// The URL to be blocked (e.g., `https://example.com/page`)
    #[serde(rename = "Url")]
    pub url: String,

    /// The date when the block was requested
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Number of days until the block expires (if applicable)
    #[serde(rename = "DaysToExpire", skip_serializing_if = "Option::is_none")]
    pub days_to_expire: Option<i32>,

    /// Whether this blocks a single page or entire directory
    #[serde(rename = "EntityType")]
    pub entity_type: BlockedUrlEntityType,

    /// Type of removal requested (cache only or full removal)
    #[serde(rename = "RequestType")]
    pub request_type: BlockedUrlRequestType,
}

/// Specifies whether a block applies to a single page or directory
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlockedUrlEntityType {
    /// Block a single page
    Page = 0,
    /// Block an entire directory and all its contents
    Directory = 1,
}

/// Type of content removal requested
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlockedUrlRequestType {
    /// Remove from cache only, keep in search results
    CacheOnly = 0,
    /// Remove completely from search results and cache
    FullRemoval = 1,
}

/// Reasons for blocking page previews
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BlockReason {
    /// Don't show preview
    NoPreview,
    /// Don't cache the page
    NoCache,
    /// Don't show snippet in search results
    NoSnippet,
    /// Don't index the page
    NoIndex,
    /// Don't show archived version
    NoArchive,
}

/// Geographic targeting settings for content
///
/// Allows you to specify which country or region specific content is targeted towards.
/// This helps Bing show the right content to users in different geographic locations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryRegionSettings {
    /// The date when this setting was configured
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Two-letter ISO country code (e.g., "US", "GB", "DE")
    #[serde(rename = "TwoLetterIsoCountryCode")]
    pub two_letter_iso_country_code: String,

    /// The scope of this geographic targeting setting
    #[serde(rename = "Type")]
    pub r#type: CountryRegionSettingsType,

    /// The URL or URL pattern this setting applies to
    #[serde(rename = "Url")]
    pub url: String,
}

/// Scope of geographic targeting
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CountryRegionSettingsType {
    /// Target a single page
    Page = 0,
    /// Target a directory and all its contents
    Directory = 1,
    /// Target the entire domain
    Domain = 2,
    /// Target a subdomain
    Subdomain = 3,
}

/// Crawl statistics for a website
///
/// Provides detailed metrics about how Bingbot crawls your website,
/// including HTTP response codes, errors, and index status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlStats {
    /// Count of pages returning other HTTP status codes not categorized below
    #[serde(rename = "AllOtherCodes")]
    pub all_other_codes: i64,

    /// Count of pages blocked by robots.txt
    #[serde(rename = "BlockedByRobotsTxt")]
    pub blocked_by_robots_txt: i64,

    /// Count of pages returning 2xx success codes
    #[serde(rename = "Code2xx")]
    pub code_2xx: i64,

    /// Count of pages returning 301 permanent redirect
    #[serde(rename = "Code301")]
    pub code_301: i64,

    /// Count of pages returning 302 temporary redirect
    #[serde(rename = "Code302")]
    pub code_302: i64,

    /// Count of pages returning 4xx client error codes
    #[serde(rename = "Code4xx")]
    pub code_4xx: i64,

    /// Count of pages returning 5xx server error codes
    #[serde(rename = "Code5xx")]
    pub code_5xx: i64,

    /// Count of connection timeouts
    #[serde(rename = "ConnectionTimeout")]
    pub connection_timeout: i64,

    /// Total number of pages crawled
    #[serde(rename = "CrawledPages")]
    pub crawled_pages: i64,

    /// Total number of crawl errors encountered
    #[serde(rename = "CrawlErrors")]
    pub crawl_errors: i64,

    /// Date of these statistics
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Count of DNS resolution failures
    #[serde(rename = "DnsFailures")]
    pub dns_failures: i64,

    /// Number of pages currently in Bing's index
    #[serde(rename = "InIndex")]
    pub in_index: i64,

    /// Number of inbound links to the site
    #[serde(rename = "InLinks")]
    pub in_links: i64,
}

/// Deep link information for search results
///
/// Deep links are additional links shown below a main search result,
/// helping users navigate directly to specific pages within your site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLink {
    /// Position of this deep link in the search results
    #[serde(rename = "Position")]
    pub position: i32,

    /// Display title for the deep link
    #[serde(rename = "Title")]
    pub title: String,

    /// URL of the deep link
    #[serde(rename = "Url")]
    pub url: String,

    /// Weight/priority of this deep link
    #[serde(rename = "Weight")]
    pub weight: DeepLinkWeight,
}

/// Priority weight for deep links
///
/// Controls how prominently a deep link should be displayed in search results.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeepLinkWeight {
    /// Deep link is disabled
    Disabled = 0,
    /// Low priority
    Low = 1,
    /// Normal priority
    Normal = 2,
    /// High priority
    High = 3,
}

/// Algorithm-suggested deep link URL
///
/// URLs that Bing's algorithm suggests as good candidates for deep links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkAlgoUrl {
    /// Number of deep links for this URL
    #[serde(rename = "DeepLinkCount")]
    pub deep_link_count: i32,

    /// Number of impressions this URL receives
    #[serde(rename = "Impressions")]
    pub impressions: i32,

    /// The suggested URL
    #[serde(rename = "Url")]
    pub url: String,
}

/// Blocked deep link
///
/// Represents a deep link that has been explicitly blocked from appearing in search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkBlock {
    /// Source URL (the main search result)
    pub source_url: String,

    /// Target URL (the deep link being blocked)
    pub target_url: String,

    /// Type of block applied
    pub block_type: String,

    /// Reason for blocking this deep link
    pub reason: String,
}

/// Page preview block
///
/// Represents a page where preview features (snippet, cache, etc.) have been blocked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagePreviewBlock {
    /// URL of the page with blocked preview
    pub url: String,

    /// Reason for blocking the preview
    pub block_reason: BlockReason,

    /// Date when the block was applied
    pub blocked_date: NaiveDate,
}

/// Content submission API quota
///
/// Daily and monthly limits for the content submission API.
/// Content submission allows submitting page content directly to Bing (up to 10MB per request).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSubmissionQuota {
    /// Daily submission quota remaining
    #[serde(rename = "DailyQuota")]
    pub daily_quota: i64,

    /// Monthly submission quota remaining
    #[serde(rename = "MonthlyQuota")]
    pub monthly_quota: i64,
}

/// Crawl rate settings for a site
///
/// Controls how frequently Bingbot crawls your site.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlSettings {
    /// Whether crawl boost feature is available for this site
    #[serde(rename = "CrawlBoostAvailable")]
    pub crawl_boost_available: bool,

    /// Whether crawl boost is currently enabled
    #[serde(rename = "CrawlBoostEnabled")]
    pub crawl_boost_enabled: bool,

    /// Crawl rate configuration data
    #[serde(rename = "CrawlRate")]
    pub crawl_rate: Vec<u8>,
}

/// Detailed query statistics for a specific date
///
/// More granular version of `QueryStats` with position data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedQueryStats {
    /// Number of clicks for this query
    #[serde(rename = "Clicks")]
    pub clicks: i64,

    /// Date of these statistics
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Number of impressions for this query
    #[serde(rename = "Impressions")]
    pub impressions: i64,

    /// Average position in search results
    #[serde(rename = "Position")]
    pub position: f64,
}

/// Search query performance statistics
///
/// Contains metrics about how a specific search query performs for your site,
/// including clicks, impressions, and ranking positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    /// Average position in search results when the page is clicked
    #[serde(rename = "AvgClickPosition")]
    pub avg_click_position: f64,

    /// Average position in search results when the page is shown (impression)
    #[serde(rename = "AvgImpressionPosition")]
    pub avg_impression_position: f64,

    /// Number of times users clicked through to your site from search results
    #[serde(rename = "Clicks")]
    pub clicks: i64,

    /// Date of these statistics
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Number of times your site appeared in search results (impressions)
    #[serde(rename = "Impressions")]
    pub impressions: i64,

    /// The search query string
    #[serde(rename = "Query")]
    pub query: String,
}

/// URL with crawl issues
///
/// Represents a URL that Bingbot encountered problems crawling,
/// along with information about the type of issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlWithCrawlIssues {
    /// HTTP status code returned when crawling this URL
    #[serde(rename = "HttpCode")]
    pub http_code: i32,

    /// Bitmask of crawl issues encountered (see CrawlIssues enum)
    #[serde(rename = "Issues")]
    pub issues: i32,

    /// The URL that has crawl issues
    #[serde(rename = "Url")]
    pub url: String,

    /// Number of inbound links pointing to this URL
    #[serde(rename = "InLinks")]
    pub in_links: i64,
}

/// RSS or Atom feed information
///
/// Contains details about a submitted feed, including its status and crawl information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    /// Whether the feed is compressed (gzipped)
    #[serde(rename = "Compressed")]
    pub compressed: bool,

    /// Size of the feed file in bytes
    #[serde(rename = "FileSize")]
    pub file_size: i64,

    /// When the feed was last crawled by Bing
    #[serde(rename = "LastCrawled", with = "dotnet_date_format_opt")]
    pub last_crawled: Option<NaiveDate>,

    /// Current status of the feed (e.g., "Active", "Pending")
    #[serde(rename = "Status")]
    pub status: String,

    /// When the feed was submitted
    #[serde(rename = "Submitted", with = "dotnet_date_format_opt")]
    pub submitted: Option<NaiveDate>,

    /// Feed type (e.g., "RSS", "Atom")
    #[serde(rename = "Type")]
    pub r#type: String,

    /// URL of the feed
    #[serde(rename = "Url")]
    pub url: String,

    /// Number of URLs contained in the feed
    #[serde(rename = "UrlCount")]
    pub url_count: i32,
}

/// Website information and verification status
///
/// Contains verification codes and status for a site in Bing Webmaster Tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    /// Authentication code for meta tag verification
    #[serde(rename = "AuthenticationCode")]
    pub authentication_code: String,

    /// DNS TXT record code for DNS verification
    #[serde(rename = "DnsVerificationCode")]
    pub dns_verification_code: String,

    /// Whether the site ownership has been verified
    #[serde(rename = "IsVerified")]
    pub is_verified: bool,

    /// The site URL (e.g., `https://example.com`)
    #[serde(rename = "Url")]
    pub url: String,
}

/// User roles and permissions for a site
///
/// Represents a user's access permissions for a specific site in Bing Webmaster Tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteRoles {
    /// Date when this role was assigned
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Delegation code for transferring site ownership
    #[serde(rename = "DelegatedCode")]
    pub delegated_code: Option<String>,

    /// Email of the user who owns the delegation code
    #[serde(rename = "DelegatedCodeOwnerEmail")]
    pub delegated_code_owner_email: Option<String>,

    /// Email of the user who delegated access
    #[serde(rename = "DelegatorEmail")]
    pub delegator_email: Option<String>,

    /// Email of the user with this role
    #[serde(rename = "Email")]
    pub email: String,

    /// Whether this role assignment has expired
    #[serde(rename = "Expired")]
    pub expired: bool,

    /// The role assigned to the user
    #[serde(rename = "Role")]
    pub role: UserRole,

    /// The site URL this role applies to
    #[serde(rename = "Site")]
    pub site: String,

    /// The verification site URL
    #[serde(rename = "VerificationSite")]
    pub verification_site: String,
}

/// User role permissions for site access
///
/// Defines the level of access a user has to a site in Bing Webmaster Tools.
///
/// Reference: Microsoft.Bing.Webmaster.Api.Interfaces.SiteRoles.UserRole
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(i32)]
pub enum UserRole {
    /// User has full administrative permissions
    Administrator = 0,
    /// User has read-only permissions
    ReadOnly = 1,
    /// User has read and write permissions
    ReadWrite = 2,
}

/// Detailed information about a specific URL
///
/// Contains comprehensive metadata about a URL including crawl status,
/// size, and link information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlInfo {
    /// Number of anchor links pointing to this URL
    #[serde(rename = "AnchorCount")]
    pub anchor_count: i32,

    /// When Bing first discovered this URL
    #[serde(rename = "DiscoveryDate")]
    pub discovery_date: NaiveDate,

    /// Size of the document in bytes
    #[serde(rename = "DocumentSize")]
    pub document_size: i64,

    /// HTTP status code returned when crawling
    #[serde(rename = "HttpStatus")]
    pub http_status: i32,

    /// Whether this is a page (true) or directory (false)
    #[serde(rename = "IsPage")]
    pub is_page: bool,

    /// When Bing last crawled this URL
    #[serde(rename = "LastCrawledDate")]
    pub last_crawled_date: NaiveDate,

    /// Total number of child URLs under this URL
    #[serde(rename = "TotalChildUrlCount")]
    pub total_child_url_count: i32,

    /// The URL
    #[serde(rename = "Url")]
    pub url: String,
}

/// Traffic statistics for a specific URL
///
/// Contains click and impression data for a URL in search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlTrafficInfo {
    /// Number of clicks this URL received
    #[serde(rename = "Clicks")]
    pub clicks: i32,

    /// Number of times this URL appeared in search results
    #[serde(rename = "Impressions")]
    pub impressions: i32,

    /// Whether this is a page (true) or directory (false)
    #[serde(rename = "IsPage")]
    pub is_page: bool,

    /// The URL
    #[serde(rename = "Url")]
    pub url: String,
}

/// URL submission API quota
///
/// Daily and monthly limits for the URL submission API.
/// Allows submitting up to 10,000 URLs per day for crawling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlSubmissionQuota {
    /// Daily URL submission quota remaining
    #[serde(rename = "DailyQuota")]
    pub daily_quota: i32,

    /// Monthly URL submission quota remaining
    #[serde(rename = "MonthlyQuota")]
    pub monthly_quota: i32,
}

/// Inbound link counts for URLs
///
/// Contains a list of URLs and how many inbound links each has.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCounts {
    /// List of URLs with their link counts
    #[serde(rename = "Links")]
    pub links: Vec<LinkCount>,

    /// Total number of pages in the result set
    #[serde(rename = "TotalPages")]
    pub total_pages: i32,
}

/// Link count for a specific URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCount {
    /// Number of inbound links to this URL
    #[serde(rename = "Count")]
    pub count: i32,

    /// The URL
    #[serde(rename = "Url")]
    pub url: String,
}

/// Detailed inbound link information
///
/// Contains specific details about inbound links including anchor text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDetails {
    /// List of detailed link information
    #[serde(rename = "Details")]
    pub details: Vec<LinkDetail>,

    /// Total number of pages in the result set
    #[serde(rename = "TotalPages")]
    pub total_pages: i32,
}

/// Detail about a specific inbound link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkDetail {
    /// The anchor text used for the link
    #[serde(rename = "AnchorText")]
    pub anchor_text: String,

    /// The source URL of the link
    #[serde(rename = "Url")]
    pub url: String,
}

/// Query parameter configuration
///
/// Represents a URL query parameter that should be ignored or included during crawling.
/// This helps prevent duplicate content issues from URL parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParameter {
    /// Date when this parameter configuration was set
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Whether this parameter is enabled (should be ignored)
    #[serde(rename = "IsEnabled")]
    pub is_enabled: bool,

    /// The query parameter name (e.g., "sessionid", "ref")
    #[serde(rename = "Parameter")]
    pub parameter: String,

    /// Source of this parameter configuration
    #[serde(rename = "Source")]
    pub source: i32,
}

/// Combined ranking and traffic statistics
///
/// Aggregated metrics for site performance in search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankAndTrafficStats {
    /// Total number of clicks
    #[serde(rename = "Clicks")]
    pub clicks: i64,

    /// Date of these statistics
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Total number of impressions
    #[serde(rename = "Impressions")]
    pub impressions: i64,
}

/// Filter properties for queries
///
/// Optional filters that can be applied when querying data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterProperties {
    /// Filter by search engine ID
    #[serde(rename = "SearchEngine")]
    pub search_engine: Option<i32>,

    /// Filter by crawl date
    #[serde(rename = "CrawlDate")]
    pub crawl_date: Option<NaiveDate>,

    /// Filter by discovered date
    #[serde(rename = "DiscoveredDate")]
    pub discovered_date: Option<NaiveDate>,
}

/// URL fetched on demand
///
/// Result of requesting Bing to fetch a specific URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedUrl {
    /// The URL that was fetched
    #[serde(rename = "Url")]
    pub url: String,

    /// When the URL was fetched
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// HTTP status code returned
    #[serde(rename = "HttpStatusCode")]
    pub http_status_code: i32,
}

/// Detailed information about a fetched URL
///
/// Extended version of `FetchedUrl` with response headers and message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedUrlDetails {
    /// The URL that was fetched
    #[serde(rename = "Url")]
    pub url: String,

    /// When the URL was fetched
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// HTTP status code returned
    #[serde(rename = "HttpStatusCode")]
    pub http_status_code: i32,

    /// HTTP response headers
    #[serde(rename = "ResponseHeaders")]
    pub response_headers: String,

    /// HTTP response message
    #[serde(rename = "HttpMessage")]
    pub http_message: String,
}

/// Keyword search statistics
///
/// Performance metrics for a specific search keyword.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    /// The search query/keyword
    #[serde(rename = "Query")]
    pub query: String,

    /// Number of times this keyword resulted in impressions
    #[serde(rename = "Impressions")]
    pub impressions: i64,

    /// Number of clicks from this keyword
    #[serde(rename = "Clicks")]
    pub clicks: i64,

    /// Date of these statistics
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,
}

/// Aggregated keyword statistics
///
/// Summary statistics for a keyword across all dates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordStats {
    /// The search query/keyword
    #[serde(rename = "Query")]
    pub query: String,

    /// Total number of impressions across all dates
    #[serde(rename = "TotalImpressions")]
    pub total_impressions: i64,

    /// Total number of clicks across all dates
    #[serde(rename = "TotalClicks")]
    pub total_clicks: i64,

    /// Average position when users click through
    #[serde(rename = "AvgClickPosition")]
    pub avg_click_position: f64,
}

/// Site migration information
///
/// Represents a site move/migration from one URL to another.
/// Used to inform Bing about domain changes or HTTPS migrations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMove {
    /// Original site URL (source)
    #[serde(rename = "SourceSite")]
    pub source_site: String,

    /// New site URL (target/destination)
    #[serde(rename = "TargetSite")]
    pub target_site: String,

    /// Date when the site move was registered
    #[serde(rename = "Date", with = "dotnet_date_format")]
    pub date: NaiveDate,

    /// Current status of the site move (e.g., "InProgress", "Complete")
    #[serde(rename = "Status")]
    pub status: String,
}

/// Site move configuration settings
///
/// Settings required to configure a site migration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMoveSettings {
    /// Target site URL (new location)
    #[serde(rename = "TargetSite")]
    pub target_site: String,

    /// Validation tag to verify ownership of target site
    #[serde(rename = "ValidationTag")]
    pub validation_tag: String,
}
