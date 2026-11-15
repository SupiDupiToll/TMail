//! Application service: owns the signed-in session, the encrypted cache and
//! the realtime event loop. This is the only type the Tauri shell talks to,
//! which keeps the core usable from a CLI or another frontend unchanged.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use base64::prelude::{Engine, BASE64_STANDARD};
use tuta_sdk::entities::generated::tutanota::Mail;
use tuta_sdk::{GeneratedId, IdTupleGenerated, LoggedInSdk};

use crate::auth::{derive_cache_key, AuthService, LoginStep, DEFAULT_API_URL};
use crate::error::{CoreError, Result};
use crate::event_bus::{EventBusClient, EventBusError, EventBusMessage, WsState};
use crate::folders::{id_tuple, FolderNode, FolderTree};
use crate::mail_ext::MailExt;
use crate::models::{AttachmentMeta, DraftInput, MailDetail, MailSummary, SearchHit, SendReceipt};
use crate::send::{send, Outgoing, OutgoingAttachment};
use crate::session::{SessionStore, StoredSession};
use crate::store::{wipe, Store};

/// Events pushed to the UI. Deliberately coarse: list contents are always
/// re-read from the cache, which keeps the event payload free of plaintext.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncEvent {
    Connected,
    Disconnected,
    MailsChanged { folder_id: String },
    SessionExpired,
    CacheWarning { message: String },
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ContactDto {
    pub name: String,
    pub company: String,
    pub addresses: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RecipientKind {
    pub address: String,
    pub internal: bool,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct FolderUnread {
    pub folder_id: String,
    pub unread: usize,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Snapshot {
    pub email: String,
    pub api_url: String,
    pub folders: Vec<FolderNode>,
    pub unread: Vec<FolderUnread>,
    pub session_persisted: bool,
    pub connected: bool,
    pub cached_mails: usize,
    pub cached_bodies: usize,
    /// Enabled sender addresses: primary first, then aliases.
    pub sender_addresses: Vec<String>,
}

pub enum LoginOutcome {
    LoggedIn(Snapshot),
    TotpRequired,
}

struct Active {
    email: String,
    sdk: Arc<LoggedInSdk>,
    mail: Arc<MailExt>,
    store: Arc<Store>,
    /// Server-authoritative unread counts by folder id. Populated by the
    /// event bus and adjusted optimistically by local actions.
    counters: Mutex<HashMap<String, usize>>,
    tree: RwLock<FolderTree>,
    access_token: String,
    user_id: String,
    display_name: String,
    sender_addresses: Vec<String>,
    session_persisted: AtomicBool,
}

impl Active {
    fn folder(&self, id: &str) -> Option<FolderNode> {
        self.tree.read().ok()?.get(id).cloned()
    }

    fn folders(&self) -> Vec<FolderNode> {
        self.tree
            .read()
            .map(|tree| tree.nodes().to_vec())
            .unwrap_or_default()
    }
}

pub struct CoreService {
    api_url: String,
    data_dir: PathBuf,
    auth: AuthService,
    sessions: SessionStore,
    active: RwLock<Option<Arc<Active>>>,
    events: tokio::sync::broadcast::Sender<SyncEvent>,
    shutdown: std::sync::Mutex<tokio::sync::watch::Sender<bool>>,
    current_folder: std::sync::Mutex<Option<String>>,
}

impl CoreService {
    pub fn new(data_dir: PathBuf, api_url: Option<String>) -> Result<Self> {
        let api_url = api_url.unwrap_or_else(|| DEFAULT_API_URL.to_string());
        let model_dir = data_dir.join("models");
        let auth = AuthService::new(&api_url, &model_dir)?;
        let (events, _) = tokio::sync::broadcast::channel(64);
        let (shutdown, _) = tokio::sync::watch::channel(false);
        Ok(Self {
            api_url,
            data_dir,
            auth,
            sessions: SessionStore::keyring(),
            active: RwLock::new(None),
            events,
            shutdown: std::sync::Mutex::new(shutdown),
            current_folder: std::sync::Mutex::new(None),
        })
    }

    #[must_use]
    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    #[must_use]
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<SyncEvent> {
        self.events.subscribe()
    }

    pub fn is_signed_in(&self) -> bool {
        self.active
            .read()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    fn active(&self) -> Result<Arc<Active>> {
        self.active
            .read()
            .ok()
            .and_then(|guard| guard.clone())
            .ok_or(CoreError::NotSignedIn)
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<LoginOutcome> {
        // Tear down any previous session before creating a new one.
        self.stop_events();
        let step = self.auth.begin(email, password).await?;
        match step {
            LoginStep::TotpRequired => Ok(LoginOutcome::TotpRequired),
            LoginStep::Done(stored) => Ok(LoginOutcome::LoggedIn(self.activate(stored).await?)),
        }
    }

    pub async fn submit_totp(&self, code: u32) -> Result<Snapshot> {
        let stored = self.auth.submit_totp(code).await?;
        self.activate(stored).await
    }

    pub async fn cancel_login(&self) {
        self.auth.cancel().await;
    }

    /// Resumes a stored session. Returns `Ok(None)` when nothing is stored so
    /// the UI can show the sign-in screen without treating it as an error.
    pub async fn resume(&self, email: &str) -> Result<Option<Snapshot>> {
        let Some(stored) = self.sessions.load(email, &self.api_url)? else {
            return Ok(None);
        };
        let snapshot = self.activate(stored).await?;
        Ok(Some(snapshot))
    }

    async fn activate(&self, stored: StoredSession) -> Result<Snapshot> {
        let sdk = self.auth.resume(&stored).await?;
        let mut key = derive_cache_key(&sdk).await?.to_vec();
        let store = Store::open(&self.cache_path(&stored.email), &stored.email, &mut key);
        wipe(&mut key);
        let store = Arc::new(store?);

        let mailbox = sdk
            .mail_facade()
            .load_user_mailbox()
            .await
            .map_err(CoreError::from)?;
        let mail = MailExt::new(Arc::clone(&sdk));
        let tree = mail.load_folders(&mailbox).await?;
        if let Ok(json) = serde_json::to_string(tree.nodes()) {
            let _ = store.save_folders(&json);
        }
        let user_id = sdk
            .get_user_id()
            .map(|id| id.0)
            .unwrap_or_else(|| stored.user_id.clone());
        let (display_name, sender_addresses) = load_account_identity(&sdk, &stored.email).await;
        self.remember_account(&stored.email);
        let session_persisted =
            self.sessions.save(&stored).is_ok() && self.sessions.is_persistent();
        if !session_persisted {
            let _ = self.events.send(SyncEvent::CacheWarning {
                message: "The OS keyring is unavailable; this session lasts until TMail closes."
                    .into(),
            });
        }

        let persisted_counters = store.load_counters().unwrap_or_default();
        let active = Arc::new(Active {
            email: stored.email.clone(),
            sdk,
            mail: Arc::new(mail),
            store,
            counters: Mutex::new(persisted_counters),
            tree: RwLock::new(tree),
            access_token: stored.access_token.clone(),
            user_id,
            display_name,
            sender_addresses,
            session_persisted: AtomicBool::new(session_persisted),
        });
        *self.active.write().unwrap() = Some(Arc::clone(&active));
        self.start_events(Arc::clone(&active));
        Ok(self.build_snapshot(&active))
    }

    fn build_snapshot(&self, active: &Active) -> Snapshot {
        let unread = merged_counts(active)
            .into_iter()
            .map(|(folder_id, unread)| FolderUnread { folder_id, unread })
            .collect();
        let (cached_mails, cached_bodies) = active.store.counts().unwrap_or((0, 0));
        Snapshot {
            email: active.email.clone(),
            api_url: self.api_url.clone(),
            folders: active.folders(),
            unread,
            session_persisted: active.session_persisted.load(Ordering::Relaxed),
            connected: false,
            cached_mails,
            cached_bodies,
            sender_addresses: active.sender_addresses.clone(),
        }
    }

    /// The last account that successfully activated. The file contains only
    /// the address and API URL, never a token; it lets a fresh WebView find the
    /// keyring entry again.
    pub fn last_account(&self) -> Option<(String, String)> {
        let raw = std::fs::read_to_string(self.data_dir.join("last-account.json")).ok()?;
        let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
        Some((
            value.get("email")?.as_str()?.to_string(),
            value.get("api_url")?.as_str()?.to_string(),
        ))
    }

    fn remember_account(&self, email: &str) {
        let path = self.data_dir.join("last-account.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let value =
            serde_json::json!({ "email": email.to_ascii_lowercase(), "api_url": self.api_url });
        if let Ok(json) = serde_json::to_vec_pretty(&value) {
            let _ = std::fs::write(&path, json);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
            }
        }
    }

    pub async fn snapshot(&self) -> Result<Snapshot> {
        let active = self.active()?;
        Ok(self.build_snapshot(&active))
    }

    pub async fn logout(&self) {
        self.stop_events();
        let email = self
            .active
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|a| a.email.clone()));
        if let Some(email) = email {
            let _ = self.sessions.delete(&email, &self.api_url);
        }
        *self.active.write().unwrap() = None;
        let _ = std::fs::remove_file(self.data_dir.join("last-account.json"));
    }

    /// Removes the encrypted cache for the current account as well as the
    /// session. Used by "remove account" in settings.
    pub async fn forget_account(&self) -> Result<()> {
        let email = self
            .active
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|a| a.email.clone()));
        self.logout().await;
        if let Some(email) = email {
            let _ = std::fs::remove_dir_all(self.cache_dir(&email));
        }
        Ok(())
    }

    pub async fn load_contacts(&self) -> Result<Vec<ContactDto>> {
        let active = self.active()?;
        let contacts = active
            .sdk
            .contact_facade
            .load_all_user_contacts()
            .await
            .map_err(CoreError::from)?;
        let mut out = contacts
            .into_iter()
            .filter_map(|contact| {
                let addresses: Vec<String> = contact
                    .mailAddresses
                    .iter()
                    .map(|address| address.address.trim().to_ascii_lowercase())
                    .filter(|address| !address.is_empty())
                    .collect();
                if addresses.is_empty() {
                    return None;
                }
                let parts = [
                    contact.title.as_deref(),
                    Some(contact.firstName.as_str()),
                    contact.middleName.as_deref(),
                    Some(contact.lastName.as_str()),
                    contact.nameSuffix.as_deref(),
                ];
                let mut name = parts
                    .into_iter()
                    .flatten()
                    .map(str::trim)
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ");
                if name.is_empty() {
                    name = contact
                        .nickname
                        .clone()
                        .filter(|value| !value.trim().is_empty())
                        .or_else(|| {
                            (!contact.company.trim().is_empty()).then(|| contact.company.clone())
                        })
                        .unwrap_or_else(|| addresses[0].clone());
                }
                Some(ContactDto {
                    name,
                    company: contact.company,
                    addresses,
                })
            })
            .collect::<Vec<_>>();
        out.sort_by_key(|contact| contact.name.to_lowercase());
        Ok(out)
    }

    pub async fn classify_recipients(&self, addresses: Vec<String>) -> Result<Vec<RecipientKind>> {
        let active = self.active()?;
        let kinds = crate::send::classify_recipients(&active.sdk, &addresses).await?;
        Ok(kinds
            .into_iter()
            .map(|(address, internal)| RecipientKind { address, internal })
            .collect())
    }

    pub fn folder_tree(&self) -> Result<Vec<FolderNode>> {
        Ok(self.active()?.folders())
    }

    pub fn unread_counts(&self) -> Result<Vec<FolderUnread>> {
        let active = self.active()?;
        Ok(merged_counts(&active)
            .into_iter()
            .map(|(folder_id, unread)| FolderUnread { folder_id, unread })
            .collect())
    }

    pub fn set_current_folder(&self, folder_id: Option<String>) {
        if let Ok(mut guard) = self.current_folder.lock() {
            *guard = folder_id;
        }
    }

    /// Reads a folder from the local cache without touching the network.
    pub fn cached_folder(
        &self,
        folder_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<MailSummary>> {
        self.active()?.store.list_folder(folder_id, limit, offset)
    }

    /// Fetches a page from the server and updates the cache.
    pub async fn refresh_folder(&self, folder_id: &str, limit: usize) -> Result<Vec<MailSummary>> {
        let active = self.active()?;
        let folder = active.folder(folder_id).ok_or(CoreError::NotFound)?;
        let mails = active.mail.load_folder_page(&folder, 0, limit).await?;
        let rows: Vec<(MailSummary, String)> = mails
            .iter()
            .map(|mail| {
                let summary = MailSummary::from_mail(mail);
                let existing = active
                    .store
                    .get_body(&summary.id)
                    .ok()
                    .flatten()
                    .map(|(_, text)| text)
                    .unwrap_or_default();
                (summary, existing)
            })
            .collect();
        active.store.upsert_mails(folder_id, &rows)?;
        active.store.list_folder(folder_id, limit, 0)
    }

    /// Returns the cached list immediately; the caller may refresh it in
    /// parallel via [`Self::refresh_folder`].
    pub async fn open_folder(&self, folder_id: &str, limit: usize) -> Result<Vec<MailSummary>> {
        self.set_current_folder(Some(folder_id.to_string()));
        if let Ok(cached) = self.cached_folder(folder_id, limit, 0) {
            if !cached.is_empty() {
                return Ok(cached);
            }
        }
        self.refresh_folder(folder_id, limit).await
    }

    /// Called by the UI after a `mails_changed` event. Cheap when nothing
    /// changed because the server page replaces the same rows.
    pub async fn refresh_current(&self, limit: usize) -> Result<Vec<MailSummary>> {
        let folder = self
            .current_folder
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .ok_or(CoreError::NotFound)?;
        self.refresh_folder(&folder, limit).await
    }

    pub async fn open_mail(&self, id: &str) -> Result<MailDetail> {
        let active = self.active()?;
        let summary = active.store.get_mail(id)?.ok_or(CoreError::NotFound)?;
        let id_tuple = parse_id(id)?;
        let mail: Mail = active
            .sdk
            .mail_facade()
            .get_crypto_entity_client()
            .load(&id_tuple)
            .await
            .map_err(CoreError::from)?;

        match active.mail.load_details(&mail).await {
            Ok(details) => {
                let attachments = active
                    .mail
                    .attachment_metas(&mail)
                    .await
                    .unwrap_or_default();
                let mut detail = MailDetail::build(&mail, &details, attachments.clone());
                if let Ok(conversation) = active
                    .sdk
                    .mail_facade()
                    .get_crypto_entity_client()
                    .load::<tuta_sdk::entities::generated::tutanota::ConversationEntry, _>(
                        &mail.conversationEntry,
                    )
                    .await
                {
                    detail.conversation_message_id = conversation.messageId;
                }
                detail.body_html =
                    inline_images(&active, &mail, &detail.body_html, &attachments).await;
                detail.body_text = crate::models::html_to_text(&detail.body_html);
                active
                    .store
                    .put_body(id, &detail.body_html, &detail.body_text)?;
                Ok(detail)
            }
            Err(error) if error.code() == "network" => {
                // Offline: serve the cached body with no attachment list.
                if let Some((html, text)) = active.store.get_body(id)? {
                    let mut detail = MailDetail {
                        summary,
                        recipients: crate::models::Recipients {
                            to: Vec::new(),
                            cc: Vec::new(),
                            bcc: Vec::new(),
                        },
                        reply_to: Vec::new(),
                        sent_ms: 0,
                        conversation_message_id: String::new(),
                        body_html: html,
                        body_text: text,
                        attachments: Vec::new(),
                        raw: None,
                    };
                    detail.summary.unread = false;
                    Ok(detail)
                } else {
                    Err(error)
                }
            }
            Err(error) => Err(error),
        }
    }

    pub async fn load_thread(&self, mail_id: &str) -> Result<Vec<MailSummary>> {
        let active = self.active()?;
        let id = parse_id(mail_id)?;
        let client = active.sdk.mail_facade().get_crypto_entity_client();
        let mail: Mail = client.load(&id).await.map_err(CoreError::from)?;
        let mut entry_id = mail.conversationEntry.clone();
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for _ in 0..50 {
            if !seen.insert(format!("{}/{}", entry_id.list_id.0, entry_id.element_id.0)) {
                break;
            }
            let entry: tuta_sdk::entities::generated::tutanota::ConversationEntry =
                match client.load(&entry_id).await {
                    Ok(entry) => entry,
                    Err(_) => break,
                };
            if let Some(thread_mail_id) = &entry.mail {
                if let Ok(thread_mail) = client.load::<Mail, _>(thread_mail_id).await {
                    let summary = MailSummary::from_mail(&thread_mail);
                    if !out
                        .iter()
                        .any(|existing: &MailSummary| existing.id == summary.id)
                    {
                        out.push(summary);
                    }
                }
            }
            let Some(previous) = entry.previous else {
                break;
            };
            entry_id = previous;
        }
        out.sort_by_key(|summary| summary.received_ms);
        Ok(out)
    }

    pub async fn download_attachment(&self, mail_id: &str, attachment_id: &str) -> Result<Vec<u8>> {
        let active = self.active()?;
        let id = parse_id(mail_id)?;
        let mail: Mail = active
            .sdk
            .mail_facade()
            .get_crypto_entity_client()
            .load(&id)
            .await
            .map_err(CoreError::from)?;
        active
            .mail
            .download_attachment(&mail, attachment_id)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))
    }

    pub async fn set_unread(&self, ids: &[String], unread: bool) -> Result<()> {
        let active = self.active()?;
        let tuples = ids
            .iter()
            .map(|id| parse_id(id))
            .collect::<Result<Vec<_>>>()?;
        active
            .sdk
            .mail_facade()
            .set_unread_status_for_mails(tuples, unread)
            .await
            .map_err(CoreError::from)?;
        for id in ids {
            let previous = active.store.get_mail(id)?.map(|mail| mail.unread);
            active.store.set_unread(id, unread)?;
            if previous == Some(unread) {
                continue;
            }
            if let Some(mail) = active.store.get_mail(id)? {
                let delta = if unread { 1isize } else { -1 };
                adjust_counters(&active, &mail.folder_ids, delta);
            }
        }
        let _ = self.events.send(SyncEvent::MailsChanged {
            folder_id: self.current_folder_name(),
        });
        Ok(())
    }

    pub async fn move_mails(&self, ids: &[String], target_folder: &str) -> Result<()> {
        let active = self.active()?;
        let target = active.folder(target_folder).ok_or(CoreError::NotFound)?;
        let tuples = ids
            .iter()
            .map(|id| parse_id(id))
            .collect::<Result<Vec<_>>>()?;
        active.mail.move_mails(tuples, id_tuple(&target)).await?;
        for id in ids {
            active.store.remove_mail(id)?;
        }
        let _ = self.events.send(SyncEvent::MailsChanged {
            folder_id: self.current_folder_name(),
        });
        Ok(())
    }

    pub async fn trash(&self, ids: &[String]) -> Result<()> {
        let active = self.active()?;
        let target = active
            .tree
            .read()
            .ok()
            .and_then(|tree| {
                tree.by_kind(crate::folders::MailSetKind::Trash)
                    .map(|f| f.id.clone())
            })
            .ok_or(CoreError::NotFound)?;
        self.move_mails(ids, &target).await
    }

    pub async fn archive(&self, ids: &[String]) -> Result<()> {
        let active = self.active()?;
        let target = active
            .tree
            .read()
            .ok()
            .and_then(|tree| {
                tree.by_kind(crate::folders::MailSetKind::Archive)
                    .map(|f| f.id.clone())
            })
            .ok_or(CoreError::NotFound)?;
        self.move_mails(ids, &target).await
    }

    pub async fn apply_labels(
        &self,
        mail_ids: Vec<String>,
        added: Vec<String>,
        removed: Vec<String>,
    ) -> Result<()> {
        let active = self.active()?;
        let mails = mail_ids
            .iter()
            .map(|id| parse_id(id))
            .collect::<Result<Vec<_>>>()?;
        let resolve = |ids: Vec<String>| -> Result<Vec<IdTupleGenerated>> {
            ids.iter()
                .map(|id| {
                    active
                        .folder(id)
                        .map(|folder| id_tuple(&folder))
                        .ok_or(CoreError::NotFound)
                })
                .collect()
        };
        let added_labels = resolve(added)?;
        let removed_labels = resolve(removed)?;
        active
            .sdk
            .get_service_executor()
            .post::<tuta_sdk::services::generated::tutanota::ApplyLabelService>(
                tuta_sdk::entities::generated::tutanota::ApplyLabelServicePostIn {
                    _format: 0,
                    mails,
                    addedLabels: added_labels,
                    removedLabels: removed_labels,
                },
                Default::default(),
            )
            .await
            .map_err(CoreError::from)?;
        Ok(())
    }

    pub async fn delete_forever(&self, ids: &[String], folder_id: &str) -> Result<()> {
        let active = self.active()?;
        let folder = active.folder(folder_id).ok_or(CoreError::NotFound)?;
        let tuples = ids
            .iter()
            .map(|id| parse_id(id))
            .collect::<Result<Vec<_>>>()?;
        active.mail.delete_mails(tuples, id_tuple(&folder)).await?;
        for id in ids {
            active.store.remove_mail(id)?;
        }
        Ok(())
    }

    pub async fn send(
        &self,
        draft: DraftInput,
        attachments: Vec<IncomingAttachment>,
    ) -> Result<SendReceipt> {
        let active = self.active()?;
        let sender = selected_sender(&active, draft.sender_address.as_deref())?;
        let outgoing = Outgoing {
            to: draft.to,
            cc: draft.cc,
            bcc: draft.bcc,
            subject: draft.subject,
            body_html: draft.body_html,
            plaintext: draft.plaintext,
            confidential: draft.confidential,
            confidential_password: draft.confidential_password,
            sender_name: active.display_name.clone(),
            sender_address: sender,
            draft_id: draft.draft_id.clone(),
            send_at_ms: draft.send_at_ms,
            own_addresses: active.sender_addresses.clone(),
            previous_message_id: draft.previous_message_id,
            conversation_type: draft.conversation_type,
            attachments: attachments
                .into_iter()
                .map(|a| OutgoingAttachment {
                    filename: a.filename,
                    mime_type: a.mime_type,
                    data: a.data,
                })
                .collect(),
        };
        let old_draft = outgoing.draft_id.clone();
        let (message_id, draft_id) = send(&active.sdk, &outgoing).await?;
        if let Some(old) = old_draft {
            let _ = self.delete_draft(&active, &old).await;
        }
        if let Some(sent) = active.tree.read().ok().and_then(|tree| {
            tree.by_kind(crate::folders::MailSetKind::Sent)
                .map(|f| f.id.clone())
        }) {
            let _ = self.refresh_folder(&sent, 100).await;
            let _ = self
                .events
                .send(SyncEvent::MailsChanged { folder_id: sent });
        }
        Ok(SendReceipt {
            message_id,
            draft_id,
        })
    }

    pub async fn save_draft(
        &self,
        draft: DraftInput,
        attachments: Vec<IncomingAttachment>,
    ) -> Result<String> {
        let active = self.active()?;
        let sender = selected_sender(&active, draft.sender_address.as_deref())?;
        let outgoing = Outgoing {
            to: draft.to,
            cc: draft.cc,
            bcc: draft.bcc,
            subject: draft.subject,
            body_html: draft.body_html,
            plaintext: draft.plaintext,
            confidential: draft.confidential,
            confidential_password: draft.confidential_password,
            sender_name: active.display_name.clone(),
            sender_address: sender,
            draft_id: draft.draft_id.clone(),
            send_at_ms: draft.send_at_ms,
            own_addresses: active.sender_addresses.clone(),
            previous_message_id: draft.previous_message_id,
            conversation_type: draft.conversation_type,
            attachments: attachments
                .into_iter()
                .map(|a| OutgoingAttachment {
                    filename: a.filename,
                    mime_type: a.mime_type,
                    data: a.data,
                })
                .collect(),
        };
        let old_draft = outgoing.draft_id.clone();
        let new_id = crate::send::save_draft(&active.sdk, &outgoing).await?;
        if let Some(old) = old_draft {
            let _ = self.delete_draft(&active, &old).await;
        }
        Ok(new_id)
    }

    async fn delete_draft(&self, active: &Active, id: &str) -> Result<()> {
        let folder = active
            .tree
            .read()
            .ok()
            .and_then(|tree| {
                tree.by_kind(crate::folders::MailSetKind::Draft)
                    .map(|f| f.id.clone())
            })
            .ok_or(CoreError::NotFound)?;
        let folder = active.folder(&folder).ok_or(CoreError::NotFound)?;
        active
            .mail
            .delete_mails(vec![parse_id(id)?], id_tuple(&folder))
            .await
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        self.active()?.store.search(query, limit)
    }

    pub fn clear_cache(&self) -> Result<()> {
        self.active()?.store.clear()
    }

    fn current_folder_name(&self) -> String {
        self.current_folder
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .unwrap_or_default()
    }

    fn cache_dir(&self, email: &str) -> PathBuf {
        let hash = crypto_primitives::blake3::blake3_hash(&[email.as_bytes()]);
        let short: String = hash[..8].iter().map(|b| format!("{b:02x}")).collect();
        self.data_dir.join("accounts").join(short)
    }

    fn cache_path(&self, email: &str) -> PathBuf {
        self.cache_dir(email).join("cache.db")
    }

    fn start_events(&self, active: Arc<Active>) {
        // A fresh shutdown channel per login generation: a stopped bus must
        // not see the flag from the previous session.
        let (shutdown, _) = tokio::sync::watch::channel(false);
        if let Ok(mut guard) = self.shutdown.lock() {
            let _ = guard.send(true);
            *guard = shutdown.clone();
        }
        let bus = Arc::new(EventBusClient::new(
            self.api_url.clone(),
            AuthService::sys_model_version(),
            AuthService::tutanota_model_version(),
            tuta_sdk::CLIENT_VERSION.to_string(),
            "TMail Desktop".into(),
        ));
        if let Ok(mut ids) = bus.last_batch_ids().lock() {
            if let Ok(cursors) = active.store.load_cursors() {
                ids.extend(cursors);
            }
        }
        let (tx, mut rx) = tokio::sync::mpsc::channel(64);
        let events = self.events.clone();
        let state_rx = bus.state();
        let token = active.access_token.clone();
        let user_id = active.user_id.clone();
        let bus_for_run = Arc::clone(&bus);
        tokio::spawn(async move {
            let result = bus_for_run
                .run(token, user_id, tx, shutdown.subscribe())
                .await;
            if let Err(EventBusError::AuthenticationRejected(_)) = result {
                let _ = events.send(SyncEvent::SessionExpired);
            }
        });
        // Publish connection state transitions.
        {
            let mut state = state_rx;
            let events = self.events.clone();
            tokio::spawn(async move {
                while state.changed().await.is_ok() {
                    match *state.borrow() {
                        WsState::Connected => {
                            let _ = events.send(SyncEvent::Connected);
                        }
                        WsState::Reconnecting | WsState::Stopped => {
                            let _ = events.send(SyncEvent::Disconnected);
                        }
                        WsState::Connecting => {}
                    }
                }
            });
        }
        let store = Arc::clone(&active.store);
        let active_events = Arc::clone(&active);
        let events = self.events.clone();
        let cursor_handle = bus.last_batch_ids();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    EventBusMessage::CounterUpdate(value) => {
                        if let Ok(json) = serde_json::to_string(&value) {
                            if let Ok(Some(counters)) = active_events
                                .mail
                                .decrypt_inline::<tuta_sdk::entities::generated::sys::WebsocketCounterData>(&json)
                                .await
                            {
                                if let Ok(mut guard) = active_events.counters.lock() {
                                    for value in counters.counterValues {
                                        let count =
                                            usize::try_from(value.count.max(0)).unwrap_or(0);
                                        guard.insert(value.counterId.0.clone(), count);
                                        let _ = active_events
                                            .store
                                            .save_counter(&value.counterId.0, count);
                                    }
                                }
                                let _ = events.send(SyncEvent::MailsChanged {
                                    folder_id: String::new(),
                                });
                            }
                        }
                    }
                    EventBusMessage::EntityUpdate(batch) => {
                        let _ = store.save_cursor(&batch.group_id, &batch.batch_id);
                        if let Ok(mut ids) = cursor_handle.lock() {
                            ids.insert(batch.group_id.clone(), batch.batch_id.clone());
                        }
                        if !batch.updates.is_empty() {
                            let _ = events.send(SyncEvent::MailsChanged {
                                folder_id: String::new(),
                            });
                        }
                    }
                    EventBusMessage::InitialSyncDone => {
                        let _ = events.send(SyncEvent::MailsChanged {
                            folder_id: String::new(),
                        });
                    }
                    _ => {}
                }
            }
        });
    }

    fn stop_events(&self) {
        if let Ok(guard) = self.shutdown.lock() {
            let _ = guard.send(true);
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct IncomingAttachment {
    pub filename: String,
    pub mime_type: String,
    #[serde(with = "attachment_bytes")]
    pub data: Vec<u8>,
}

mod attachment_bytes {
    use base64::prelude::{Engine, BASE64_STANDARD};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let encoded = String::deserialize(deserializer)?;
        BASE64_STANDARD
            .decode(encoded)
            .map_err(serde::de::Error::custom)
    }
}

fn parse_id(id: &str) -> Result<IdTupleGenerated> {
    let (list, element) = id
        .split_once('/')
        .ok_or_else(|| CoreError::InvalidInput("mail id must be list/element".into()))?;
    if list.is_empty() || element.is_empty() {
        return Err(CoreError::InvalidInput(
            "mail id must be list/element".into(),
        ));
    }
    Ok(IdTupleGenerated {
        list_id: GeneratedId(list.to_string()),
        element_id: GeneratedId(element.to_string()),
    })
}

async fn load_account_identity(sdk: &Arc<LoggedInSdk>, email: &str) -> (String, Vec<String>) {
    let user = sdk.get_user();
    let group_info_id = user.userGroup.groupInfo.clone();
    let info = sdk
        .mail_facade()
        .get_crypto_entity_client()
        .load::<tuta_sdk::entities::generated::sys::GroupInfo, _>(&group_info_id)
        .await
        .ok();

    let name = info
        .as_ref()
        .map(|info| info.name.clone())
        .filter(|name| !name.is_empty())
        .unwrap_or_default();

    let mut addresses = Vec::new();
    if let Some(info) = &info {
        if let Some(primary) = &info.mailAddress {
            addresses.push(primary.to_ascii_lowercase());
        }
        for alias in &info.mailAddressAliases {
            if alias.enabled {
                addresses.push(alias.mailAddress.to_ascii_lowercase());
            }
        }
    }
    let primary = email.to_ascii_lowercase();
    if !addresses.iter().any(|address| address == &primary) {
        addresses.insert(0, primary);
    }
    addresses.dedup();
    (name, addresses)
}

/// Replaces `cid:` image references with data URLs for small inline
/// attachments, so a message renders its signature/logo without network access.
async fn inline_images(
    active: &Active,
    mail: &Mail,
    html: &str,
    attachments: &[AttachmentMeta],
) -> String {
    use base64::prelude::{Engine, BASE64_STANDARD};
    let mut out = html.to_string();
    for attachment in attachments {
        let Some(cid) = attachment.cid.as_deref() else {
            continue;
        };
        let token = format!("cid:{cid}");
        if !out.contains(&token) {
            continue;
        }
        if attachment.size > 2 * 1024 * 1024 {
            log::debug!(
                "inline image {} is too large; leaving it blocked",
                attachment.name
            );
            continue;
        }
        match active.mail.download_attachment(mail, &attachment.id).await {
            Ok(bytes) => {
                let encoded = BASE64_STANDARD.encode(bytes);
                let data_url = format!("data:{};base64,{encoded}", attachment.mime_type);
                out = out.replace(&token, &data_url);
            }
            Err(error) => log::debug!("inline image {} unavailable: {error}", attachment.name),
        }
    }
    out
}

fn selected_sender(active: &Active, requested: Option<&str>) -> Result<String> {
    let requested = requested
        .unwrap_or(&active.email)
        .trim()
        .to_ascii_lowercase();
    matching_address(&active.sender_addresses, &requested).ok_or_else(|| {
        CoreError::InvalidInput("this sender address is not enabled for the account".into())
    })
}

fn matching_address(addresses: &[String], wanted: &str) -> Option<String> {
    addresses
        .iter()
        .find(|address| address.eq_ignore_ascii_case(wanted))
        .cloned()
}

fn merged_counts(active: &Active) -> HashMap<String, usize> {
    let mut counts = active.store.unread_counts().unwrap_or_default();
    if let Ok(guard) = active.counters.lock() {
        for (folder, count) in guard.iter() {
            counts.insert(folder.clone(), *count);
        }
    }
    counts
}

fn adjust_counters(active: &Active, folder_ids: &[String], delta: isize) {
    if let Ok(mut guard) = active.counters.lock() {
        for folder in folder_ids {
            let current = guard.get(folder).copied().unwrap_or(0);
            let next = (current as isize + delta).max(0) as usize;
            guard.insert(folder.clone(), next);
            let _ = active.store.save_counter(folder, next);
        }
    }
}

/// Cross-platform per-user data directory (XDG on Linux, LOCALAPPDATA on
/// Windows, Application Support on macOS).
#[must_use]
pub fn default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .map(|dir| dir.join("tmail"))
        .unwrap_or_else(|| PathBuf::from(".").join("tmail-data"))
}

/// Encodes an attachment for JSON transport to the UI. The UI receives bytes
/// only for the file the user explicitly saved.
#[must_use]
pub fn attachment_to_base64(data: &[u8]) -> String {
    BASE64_STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mail_ids_strictly() {
        assert!(parse_id("list/element").is_ok());
        assert!(parse_id("nope").is_err());
        assert!(parse_id("/element").is_err());
        assert!(parse_id("list/").is_err());
    }

    #[test]
    fn data_dir_is_absolute_or_local() {
        let dir = default_data_dir();
        assert!(!dir.as_os_str().is_empty());
    }
}
