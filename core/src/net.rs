//! Network plumbing: one HTTP client, bounded concurrency, backoff and a
//! Tuta-only URL allowlist.
//!
//! The SDK can inject a REST client; we wrap the native hyper client so every
//! request made by the SDK goes through the same choke point. This is also
//! where "no network calls except to Tuta" is enforced.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tuta_sdk::bindings::rest_client::{
    HttpMethod, RestClient, RestClientError, RestClientOptions, RestResponse,
};
use tuta_sdk::net::native_rest_client::NativeRestClient;

/// Concurrency cap. Tuta tolerates bursts, but a desktop client has no reason
/// to open more than a handful of sockets; this keeps latency predictable and
/// makes the client a good API citizen.
const MAX_CONCURRENT: usize = 4;
const MAX_ATTEMPTS: u32 = 3;
/// If Tuta omits `Retry-After`, pause all requests for this long after a 429.
const DEFAULT_RATE_LIMIT_COOLDOWN: Duration = Duration::from_secs(10);
const MAX_RATE_LIMIT_COOLDOWN_SECS: u64 = 300;

/// Domains owned by Tutao. The API host is derived from configuration; blob
/// storage may live on a different Tuta host, so the owned domains are allowed
/// as well. Nothing else is reachable, including redirects to third parties.
const TUTA_DOMAINS: &[&str] = &["tuta.com", "tutanota.com", "tuta.io", "tutanota.de"];

pub struct GovernedRestClient {
    inner: NativeRestClient,
    allowed_scheme: String,
    allowed_host: String,
    gate: tokio::sync::Semaphore,
    /// Shared cooldown so a 429 from one request stops the whole client from
    /// hammering the account with parallel retries.
    rate_limit_until: Arc<tokio::sync::Mutex<Option<Instant>>>,
    client_label: String,
}

impl GovernedRestClient {
    pub fn new(api_url: &str, client_label: &str) -> Result<Self, RestClientError> {
        let (allowed_scheme, allowed_host) =
            split_url(api_url).ok_or(RestClientError::InvalidURL)?;
        let inner = NativeRestClient::try_new().map_err(|_| RestClientError::FailedTlsSetup)?;
        Ok(Self {
            inner,
            allowed_scheme,
            allowed_host,
            gate: tokio::sync::Semaphore::new(MAX_CONCURRENT),
            rate_limit_until: Arc::new(tokio::sync::Mutex::new(None)),
            client_label: client_label.to_string(),
        })
    }

    fn is_allowed(&self, url: &str) -> bool {
        let Some((scheme, host)) = split_url(url) else {
            return false;
        };
        if scheme != self.allowed_scheme {
            return false;
        }
        if host == self.allowed_host {
            return true;
        }
        TUTA_DOMAINS.iter().any(|domain| {
            host == *domain
                || host
                    .strip_suffix(domain)
                    .is_some_and(|prefix| prefix.ends_with('.'))
        })
    }

    async fn wait_for_rate_limit(&self) {
        loop {
            let until = { *self.rate_limit_until.lock().await };
            let Some(until) = until else {
                return;
            };
            let now = Instant::now();
            if until <= now {
                *self.rate_limit_until.lock().await = None;
                return;
            }
            tokio::time::sleep(until - now).await;
        }
    }

    async fn note_rate_limit(&self, response: &RestResponse) {
        let seconds = response
            .headers
            .get("retry-after")
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_RATE_LIMIT_COOLDOWN.as_secs())
            .clamp(1, MAX_RATE_LIMIT_COOLDOWN_SECS);
        let until = Instant::now() + Duration::from_secs(seconds);
        let mut guard = self.rate_limit_until.lock().await;
        if guard.is_none_or(|current| until > current) {
            *guard = Some(until);
        }
        log::warn!("Tuta rate limit (429); pausing requests for {seconds}s");
    }

    async fn backoff(attempt: u32) {
        let base = 250u64 << attempt.min(5);
        let jitter = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::from(d.subsec_nanos()))
            .unwrap_or(0))
            % 250;
        tokio::time::sleep(Duration::from_millis(base + jitter)).await;
    }
}

#[async_trait]
impl RestClient for GovernedRestClient {
    async fn request_binary(
        &self,
        url: String,
        method: HttpMethod,
        mut options: RestClientOptions,
    ) -> Result<RestResponse, RestClientError> {
        if !self.is_allowed(&url) {
            log::error!(
                "refused non-Tuta request to host {:?}",
                split_url(&url).map(|(_, host)| host)
            );
            return Err(RestClientError::InvalidURL);
        }
        // Identify the unofficial client on every request. The SDK already
        // sets `cv`; this adds a stable product marker.
        options
            .headers
            .entry("user-agent".to_string())
            .or_insert_with(|| self.client_label.clone());

        let _permit = self
            .gate
            .acquire()
            .await
            .map_err(|_| RestClientError::Unknown)?;
        let mut attempt = 0;
        loop {
            self.wait_for_rate_limit().await;
            let result = self
                .inner
                .request_binary(url.clone(), clone_method(&method), clone_options(&options))
                .await;
            match result {
                Ok(response) if response.status == 429 => {
                    // Let the SDK's SuspendableRestClient see the original 429
                    // and Retry-After header; do not add our own retry storm.
                    self.note_rate_limit(&response).await;
                    return Ok(response);
                }
                Ok(response)
                    if matches!(response.status, 502..=504) && attempt + 1 < MAX_ATTEMPTS =>
                {
                    attempt += 1;
                    log::warn!("Tuta returned {}; backing off", response.status);
                    Self::backoff(attempt).await;
                }
                other => return other,
            }
        }
    }
}

fn clone_method(method: &HttpMethod) -> HttpMethod {
    match method {
        HttpMethod::GET => HttpMethod::GET,
        HttpMethod::POST => HttpMethod::POST,
        HttpMethod::PUT => HttpMethod::PUT,
        HttpMethod::DELETE => HttpMethod::DELETE,
    }
}

fn clone_options(options: &RestClientOptions) -> RestClientOptions {
    RestClientOptions {
        headers: options.headers.clone(),
        body: options.body.clone(),
        suspension_behavior: options.suspension_behavior.clone(),
    }
}

fn split_url(url: &str) -> Option<(String, String)> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "https" && scheme != "http" {
        return None;
    }
    let authority = rest.split('/').next()?;
    // Reject userinfo: `https://tuta.com@evil.example/` must not pass.
    if authority.contains('@') {
        return None;
    }
    let host = authority.split(':').next()?;
    if host.is_empty() || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    Some((scheme, host.to_ascii_lowercase()))
}

/// Type-model cache directory for the SDK. Only schema JSON lives here — no
/// account data.
pub fn file_client(
    app_dir: &std::path::Path,
) -> Arc<dyn tuta_sdk::bindings::file_client::FileClient> {
    match tuta_sdk::bindings::native_file_client::NativeFileClient::try_new(app_dir.to_path_buf()) {
        Ok(client) => Arc::new(client),
        Err(error) => {
            log::error!("could not prepare SDK model directory: {error}");
            Arc::new(FallbackFileClient {
                dir: app_dir.to_path_buf(),
            })
        }
    }
}

struct FallbackFileClient {
    dir: std::path::PathBuf,
}

#[async_trait]
impl tuta_sdk::bindings::file_client::FileClient for FallbackFileClient {
    async fn persist_content(
        &self,
        name: String,
        content: Vec<u8>,
    ) -> Result<(), tuta_sdk::bindings::file_client::FileClientError> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|_| tuta_sdk::bindings::file_client::FileClientError::IoError)?;
        std::fs::write(self.dir.join(name), content)
            .map_err(|_| tuta_sdk::bindings::file_client::FileClientError::IoError)
    }

    async fn read_content(
        &self,
        name: String,
    ) -> Result<Vec<u8>, tuta_sdk::bindings::file_client::FileClientError> {
        std::fs::read(self.dir.join(name))
            .map_err(|_| tuta_sdk::bindings::file_client::FileClientError::NotFound)
    }
}

#[allow(dead_code)]
fn _assert_headers_are_lowercase(headers: &HashMap<String, String>) -> bool {
    headers.keys().all(|k| k == &k.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_tuta_hosts_are_allowed() {
        let client = GovernedRestClient::new("https://app.tuta.com", "TMail/0.1").unwrap();
        assert!(client.is_allowed("https://app.tuta.com/rest/sys/saltservice"));
        assert!(client.is_allowed("https://app.tuta.com"));
        assert!(client.is_allowed("https://blob1.tuta.com/blob"));
        assert!(client.is_allowed("https://blob.tutanota.com/blob"));
        assert!(client.is_allowed("https://tuta.io/x"));
        assert!(!client.is_allowed("https://evil.example/collect"));
        assert!(!client.is_allowed("https://tuta.com.evil.example/x"));
        assert!(!client.is_allowed("https://app.tuta.com@evil.example/x"));
        assert!(!client.is_allowed("http://app.tuta.com/x"));
        assert!(!client.is_allowed("https://127.0.0.1/x"));
        assert!(!client.is_allowed("ftp://app.tuta.com/x"));
    }

    #[test]
    fn host_parser_handles_ports_and_userinfo() {
        assert_eq!(
            split_url("https://App.Tuta.com:443/x")
                .map(|(_, host)| host)
                .as_deref(),
            Some("app.tuta.com")
        );
        assert_eq!(
            split_url("https://app.tuta.com")
                .map(|(_, host)| host)
                .as_deref(),
            Some("app.tuta.com")
        );
        assert_eq!(split_url("https://a@b.example/x"), None);
        assert_eq!(split_url("nonsense"), None);
    }
}
