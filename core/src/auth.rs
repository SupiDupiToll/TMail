//! Sign-in, TOTP second factor and session resume.
//!
//! The SDK keeps all cryptography; this module only sequences the official
//! calls. The account password is used once to derive the passphrase key and
//! is never stored or logged.

use std::sync::Arc;

use tuta_sdk::login::Credentials;
use tuta_sdk::tutanota_constants::SecondFactorType;
use tuta_sdk::{LoggedInSdk, Sdk};

use crate::error::{CoreError, Result};
use crate::net;
use crate::session::StoredSession;

pub const DEFAULT_API_URL: &str = "https://app.tuta.com";
pub const CLIENT_IDENTIFIER: &str = "TMail Desktop";

const SYS_MODEL: &str =
    include_str!("../../vendor/tuta/tuta-sdk/rust/sdk/src/type_models/sys.json");
const TUTANOTA_MODEL: &str =
    include_str!("../../vendor/tuta/tuta-sdk/rust/sdk/src/type_models/tutanota.json");

struct PendingLogin {
    email: String,
    credentials: Credentials,
}

pub enum LoginStep {
    Done(StoredSession),
    TotpRequired,
}

pub struct AuthService {
    sdk: Arc<Sdk>,
    api_url: String,
    pending: tokio::sync::Mutex<Option<PendingLogin>>,
}

impl AuthService {
    /// `app_dir` holds only the SDK's type-model cache.
    pub fn new(api_url: &str, app_dir: &std::path::Path) -> Result<Self> {
        std::fs::create_dir_all(app_dir).map_err(|_| CoreError::Storage)?;
        let rest = Arc::new(net::GovernedRestClient::new(
            api_url,
            "TMail/0.1 (unofficial)",
        )?);
        let files = net::file_client(app_dir);
        let sdk = Arc::new(Sdk::new(api_url.to_string(), rest, files));
        Ok(Self {
            sdk,
            api_url: api_url.trim_end_matches('/').to_string(),
            pending: tokio::sync::Mutex::new(None),
        })
    }

    #[must_use]
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    /// Starts a fresh login. If the account needs TOTP, the session is parked
    /// and [`Self::submit_totp`] must be called next.
    pub async fn begin(&self, email: &str, password: &str) -> Result<LoginStep> {
        let email = email.trim().to_ascii_lowercase();
        if !email.contains('@') {
            return Err(CoreError::InvalidInput("enter a mail address".into()));
        }
        let session = self
            .sdk
            .initiate_session(&email, password, CLIENT_IDENTIFIER)
            .await
            .map_err(CoreError::from)?;

        if session.challenges.is_empty() {
            let stored =
                StoredSession::from_credentials(&email, &self.api_url, &session.credentials);
            // Verify the session once before reporting success, so a corrupt
            // server response does not get persisted.
            let _ = self
                .sdk
                .login(session.credentials)
                .await
                .map_err(CoreError::from)?;
            return Ok(LoginStep::Done(stored));
        }

        let has_totp = session
            .challenges
            .iter()
            .any(|c| c.r#type == i64::from(SecondFactorType::Totp));
        if !has_totp {
            return Err(CoreError::UnsupportedSecondFactor);
        }
        *self.pending.lock().await = Some(PendingLogin {
            email,
            credentials: session.credentials,
        });
        Ok(LoginStep::TotpRequired)
    }

    /// Submits a TOTP code and waits until the server marks the session
    /// approved (the server may take a moment after the code is accepted).
    pub async fn submit_totp(&self, code: u32) -> Result<StoredSession> {
        let pending = self
            .pending
            .lock()
            .await
            .take()
            .ok_or(CoreError::SecondFactorRequired)?;
        self.sdk
            .authenticate_with_second_factor_totp(&pending.credentials.access_token, code)
            .await
            .map_err(|_| CoreError::BadCredentials)?;

        let mut approved = false;
        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            match self
                .sdk
                .is_second_factor_pending(&pending.credentials.access_token)
                .await
            {
                Ok(false) => {
                    approved = true;
                    break;
                }
                Ok(true) => {}
                // Network hiccup during polling should not discard the code.
                Err(error) => log::debug!("2FA poll failed: {error}"),
            }
        }
        if !approved {
            return Err(CoreError::TemporarilyUnavailable);
        }
        let stored =
            StoredSession::from_credentials(&pending.email, &self.api_url, &pending.credentials);
        self.sdk
            .login(pending.credentials)
            .await
            .map_err(CoreError::from)?;
        Ok(stored)
    }

    pub async fn cancel(&self) {
        *self.pending.lock().await = None;
    }

    pub fn resume(
        &self,
        session: &StoredSession,
    ) -> impl std::future::Future<Output = Result<Arc<LoggedInSdk>>> {
        let credentials = session.to_credentials();
        let sdk = Arc::clone(&self.sdk);
        async move {
            let credentials = credentials?;
            sdk.login(credentials).await.map_err(CoreError::from)
        }
    }

    #[must_use]
    pub fn sys_model_version() -> u32 {
        parse_model_version(SYS_MODEL)
    }

    #[must_use]
    pub fn tutanota_model_version() -> u32 {
        parse_model_version(TUTANOTA_MODEL)
    }
}

fn parse_model_version(json: &str) -> u32 {
    let value: serde_json::Value =
        serde_json::from_str(json).expect("bundled type model must be valid JSON");
    value
        .as_object()
        .and_then(|map| map.values().next())
        .and_then(|first| first.get("version"))
        .and_then(serde_json::Value::as_u64)
        .map_or(0, |v| u32::try_from(v).unwrap_or(u32::MAX))
}

/// Derives the SQLCipher passphrase from the user's group key. The key never
/// leaves the process; the derivation is BLAKE3 in KDF mode.
pub async fn derive_cache_key(sdk: &LoggedInSdk) -> Result<zeroize::Zeroizing<Vec<u8>>> {
    let user_group = sdk.get_user_group_id();
    let key = sdk
        .get_current_sym_group_key(&user_group)
        .await
        .map_err(|_| CoreError::CacheUnavailable)?;
    let derived =
        crypto_primitives::blake3::blake3_kdf(&[key.object.as_bytes()], "tmail local cache v1", 32);
    Ok(zeroize::Zeroizing::new(derived))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_versions_are_positive() {
        assert!(AuthService::sys_model_version() > 0);
        assert!(AuthService::tutanota_model_version() > 0);
    }
}
