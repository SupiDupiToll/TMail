//! Session persistence.
//!
//! Only the Tuta *session* is stored — access token plus the encrypted
//! passphrase key — never the account password. On Linux the Secret Service
//! (gnome-keyring / KWallet) holds one JSON entry per account. If no keyring
//! is available the app still works in memory and tells the user that the
//! session will not survive a restart; it never falls back to plaintext disk
//! storage.

use std::sync::{Arc, Mutex};

use base64::prelude::{Engine, BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use tuta_sdk::login::{CredentialType, Credentials};
use tuta_sdk::GeneratedId;
use zeroize::Zeroize;

use crate::error::{CoreError, Result};

const SERVICE: &str = "de.tmail.TMail";

#[derive(Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub email: String,
    pub api_url: String,
    pub user_id: String,
    pub access_token: String,
    /// Base64 of the encrypted passphrase key.
    pub encrypted_passphrase_key: String,
    pub credential_type: String,
    pub created_at_ms: i64,
}

impl std::fmt::Debug for StoredSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the token or key.
        f.debug_struct("StoredSession")
            .field("email", &self.email)
            .field("api_url", &self.api_url)
            .field("user_id", &self.user_id)
            .field("created_at_ms", &self.created_at_ms)
            .finish_non_exhaustive()
    }
}

impl Drop for StoredSession {
    fn drop(&mut self) {
        self.access_token.zeroize();
        self.encrypted_passphrase_key.zeroize();
    }
}

impl StoredSession {
    #[must_use]
    pub fn from_credentials(email: &str, api_url: &str, credentials: &Credentials) -> Self {
        Self {
            email: email.to_ascii_lowercase(),
            api_url: api_url.trim_end_matches('/').to_string(),
            user_id: credentials.user_id.0.clone(),
            access_token: credentials.access_token.clone(),
            encrypted_passphrase_key: BASE64_STANDARD.encode(&credentials.encrypted_passphrase_key),
            credential_type: match credentials.credential_type {
                CredentialType::Internal => "internal".into(),
                CredentialType::External => "external".into(),
            },
            created_at_ms: now_ms(),
        }
    }

    pub fn to_credentials(&self) -> Result<Credentials> {
        let encrypted = BASE64_STANDARD
            .decode(&self.encrypted_passphrase_key)
            .map_err(|_| CoreError::SessionInvalid)?;
        Ok(Credentials {
            login: self.email.clone(),
            user_id: GeneratedId(self.user_id.clone()),
            access_token: self.access_token.clone(),
            encrypted_passphrase_key: encrypted,
            credential_type: match self.credential_type.as_str() {
                "external" => CredentialType::External,
                _ => CredentialType::Internal,
            },
        })
    }
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Storage backends are swappable so tests never touch a real keyring.
pub trait SessionBackend: Send + Sync {
    fn load(&self, account: &str) -> Result<Option<String>>;
    fn save(&self, account: &str, value: &str) -> Result<()>;
    fn delete(&self, account: &str) -> Result<()>;
}

/// Secret Service / KWallet on Linux, with the same code path on other
/// platforms once a backend is added.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub struct KeyringBackend;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
impl SessionBackend for KeyringBackend {
    fn load(&self, account: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(SERVICE, account).map_err(|e| {
            log::warn!("keyring entry unavailable: {}", e);
            CoreError::KeyringUnavailable
        })?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => {
                log::warn!("keyring read failed: {}", e);
                Err(CoreError::KeyringUnavailable)
            }
        }
    }

    fn save(&self, account: &str, value: &str) -> Result<()> {
        let entry =
            keyring::Entry::new(SERVICE, account).map_err(|_| CoreError::KeyringUnavailable)?;
        entry.set_password(value).map_err(|e| {
            log::warn!("keyring write failed: {}", e);
            CoreError::KeyringUnavailable
        })
    }

    fn delete(&self, account: &str) -> Result<()> {
        let entry =
            keyring::Entry::new(SERVICE, account).map_err(|_| CoreError::KeyringUnavailable)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(CoreError::KeyringUnavailable),
        }
    }
}

/// Volatile backend used when the keyring is missing and in tests.
#[derive(Default, Clone)]
pub struct MemoryBackend {
    entries: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl SessionBackend for MemoryBackend {
    fn load(&self, account: &str) -> Result<Option<String>> {
        Ok(self.entries.lock().unwrap().get(account).cloned())
    }

    fn save(&self, account: &str, value: &str) -> Result<()> {
        self.entries
            .lock()
            .unwrap()
            .insert(account.to_string(), value.to_string());
        Ok(())
    }

    fn delete(&self, account: &str) -> Result<()> {
        self.entries.lock().unwrap().remove(account);
        Ok(())
    }
}

pub struct SessionStore {
    backend: Arc<dyn SessionBackend>,
    persistent: bool,
}

impl SessionStore {
    #[must_use]
    pub fn keyring() -> Self {
        Self {
            backend: Arc::new(KeyringBackend),
            persistent: true,
        }
    }

    #[must_use]
    pub fn memory() -> Self {
        Self {
            backend: Arc::new(MemoryBackend::default()),
            persistent: false,
        }
    }

    #[cfg(test)]
    #[must_use]
    pub fn with_backend(backend: Arc<dyn SessionBackend>) -> Self {
        Self {
            backend,
            persistent: true,
        }
    }

    #[must_use]
    pub fn is_persistent(&self) -> bool {
        self.persistent
    }

    fn account(email: &str, api_url: &str) -> String {
        format!("{}@{}", email.to_ascii_lowercase(), api_url)
    }

    pub fn load(&self, email: &str, api_url: &str) -> Result<Option<StoredSession>> {
        let Some(raw) = self.backend.load(&Self::account(email, api_url))? else {
            return Ok(None);
        };
        let session: StoredSession = serde_json::from_str(&raw).map_err(|_| {
            log::warn!("saved session is unreadable");
            CoreError::SessionInvalid
        })?;
        Ok(Some(session))
    }

    pub fn save(&self, session: &StoredSession) -> Result<()> {
        let raw = serde_json::to_string(session).map_err(|_| CoreError::Storage)?;
        self.backend
            .save(&Self::account(&session.email, &session.api_url), &raw)
    }

    pub fn delete(&self, email: &str, api_url: &str) -> Result<()> {
        self.backend.delete(&Self::account(email, api_url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credentials() -> Credentials {
        Credentials {
            login: "me@tuta.com".into(),
            user_id: GeneratedId("user".into()),
            access_token: "secret-token".into(),
            encrypted_passphrase_key: vec![1, 2, 3],
            credential_type: CredentialType::Internal,
        }
    }

    #[test]
    fn round_trip_keeps_secrets_out_of_debug() {
        let stored =
            StoredSession::from_credentials("Me@Tuta.com", "https://app.tuta.com", &credentials());
        let debug = format!("{stored:?}");
        assert!(!debug.contains("secret-token"));
        assert!(debug.contains("me@tuta.com"));

        let restored = stored.to_credentials().unwrap();
        assert_eq!(restored.access_token, "secret-token");
        assert_eq!(restored.encrypted_passphrase_key, vec![1, 2, 3]);
    }

    #[test]
    fn store_round_trip_and_delete() {
        let store = SessionStore::with_backend(Arc::new(MemoryBackend::default()));
        assert!(store
            .load("me@tuta.com", "https://app.tuta.com")
            .unwrap()
            .is_none());
        let stored =
            StoredSession::from_credentials("me@tuta.com", "https://app.tuta.com", &credentials());
        store.save(&stored).unwrap();
        let loaded = store
            .load("me@tuta.com", "https://app.tuta.com")
            .unwrap()
            .unwrap();
        assert_eq!(loaded.user_id, "user");
        store.delete("me@tuta.com", "https://app.tuta.com").unwrap();
        assert!(store
            .load("me@tuta.com", "https://app.tuta.com")
            .unwrap()
            .is_none());
    }
}
