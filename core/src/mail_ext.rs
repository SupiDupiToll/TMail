//! Mail operations on top of the Rust SDK.
//!
//! Adapted from `spartanz51/tutabridge` (`crates/tuta/src/mail.rs`,
//! GPL-3.0) and reworked: attachment metadata and attachment payloads are
//! separate operations so opening a message never downloads megabytes, and
//! errors carry user-safe classifications. See `NOTICE.md`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crypto_primitives::key::GenericAesKey;
use crypto_primitives::randomizer_facade::RandomizerFacade;
use serde::de::DeserializeOwned;
use tuta_sdk::crypto::crypto_facade::{BucketKeyError, ResolvedSessionKey};
use tuta_sdk::entities::entity_facade::{
    has_kdf_nonce, has_owner_enc_session_key, list_element_id, owner_group,
    EntityDecryptionRequirements, EntityFacade, EntityFacadeImpl,
};
use tuta_sdk::entities::generated::tutanota::{
    DeleteMailData, Mail, MailDetails, MailDetailsBlob, MailDetailsDraft, MailSet, MoveMailData,
    TutanotaFile,
};
use tuta_sdk::entities::Entity;
use tuta_sdk::services::generated::tutanota::{MailService, MoveMailService};
use tuta_sdk::tutanota_constants::ArchiveDataType;
use tuta_sdk::{ApiCallError, GeneratedId, IdTupleGenerated, ListLoadDirection, LoggedInSdk};

use crate::error::{CoreError, Result};
use crate::folders::{FolderNode, FolderTree};
use crate::models::AttachmentMeta;

const MAIL_BATCH_LIMIT: usize = 50;

/// Why an attachment could not be read. Deterministic contract refusals are
/// separated from transport failures so the UI can decide whether retrying
/// makes sense.
#[derive(Debug, thiserror::Error)]
pub enum AttachmentError {
    #[error("{0}")]
    Contract(#[from] ContractError),
    #[error("the attachment could not be fetched from the server")]
    Transport(#[source] ApiCallError),
    #[error("the attachment could not be read")]
    Indeterminate(#[source] ApiCallError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ContractError {
    #[error("a loaded File is not one of the mail's attachments")]
    NotInMailAttachments,
    #[error("the server did not return every requested File")]
    IncompleteBatch,
    #[error("a File was returned or referenced twice")]
    DuplicateInBatch,
    #[error("the server returned a File from another list")]
    WrongList,
    #[error("the File has no key of its own and the mail carries no bucket")]
    KeyUnavailable,
    #[error("the mail's bucket has no entry for the File")]
    NoBucketEntry,
    #[error("the mail's bucket has more than one entry for the File")]
    AmbiguousBucketEntries,
    #[error("the mail's bucket uses an unsupported group reference")]
    GroupReferenceUnsupported,
    #[error("the File and the mail belong to different groups")]
    UnsupportedGroupContext,
}

fn classify(error: ApiCallError) -> AttachmentError {
    match error {
        ApiCallError::RestClient { .. } | ApiCallError::ServerResponseError { .. } => {
            AttachmentError::Transport(error)
        }
        ApiCallError::InternalSdkError { .. } => AttachmentError::Indeterminate(error),
    }
}

fn classify_bucket(error: BucketKeyError) -> AttachmentError {
    match error {
        BucketKeyError::NoEntry => ContractError::NoBucketEntry.into(),
        BucketKeyError::AmbiguousEntries => ContractError::AmbiguousBucketEntries.into(),
        BucketKeyError::GroupReferenceUnsupported => {
            ContractError::GroupReferenceUnsupported.into()
        }
        BucketKeyError::Resolution(error) => {
            AttachmentError::Indeterminate(ApiCallError::internal(error.to_string()))
        }
    }
}

fn pair(id: &IdTupleGenerated) -> (GeneratedId, GeneratedId) {
    (id.list_id.clone(), id.element_id.clone())
}

fn id_key(id: &IdTupleGenerated) -> String {
    format!("{}/{}", id.list_id.0, id.element_id.0)
}

enum BodySource<'a> {
    Blob(&'a IdTupleGenerated),
    Draft(&'a IdTupleGenerated),
}

/// Mail operations for the logged-in session.
pub struct MailExt {
    sdk: Arc<LoggedInSdk>,
    decoder: EntityFacadeImpl,
    cache: tokio::sync::Mutex<DetailCache>,
}

#[derive(Default)]
struct DetailCache {
    /// Full details are small (bodies are separate blobs) and are useful when
    /// the same message is re-opened. Bounded by insertion order.
    entries: Vec<(String, MailDetails)>,
}

impl DetailCache {
    const MAX: usize = 64;

    fn get(&self, key: &str) -> Option<MailDetails> {
        self.entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn put(&mut self, key: String, value: MailDetails) {
        if self.entries.iter().any(|(k, _)| k == &key) {
            return;
        }
        if self.entries.len() >= Self::MAX {
            self.entries.remove(0);
        }
        self.entries.push((key, value));
    }
}

impl MailExt {
    #[must_use]
    pub fn new(sdk: Arc<LoggedInSdk>) -> Self {
        let decoder = EntityFacadeImpl::new(
            sdk.type_model_provider.clone(),
            RandomizerFacade::from_core(rand_core::OsRng),
        );
        Self {
            sdk,
            decoder,
            cache: tokio::sync::Mutex::new(DetailCache::default()),
        }
    }

    pub async fn load_folders(
        &self,
        mailbox: &tuta_sdk::entities::generated::tutanota::MailBox,
    ) -> Result<FolderTree> {
        let folders: Vec<MailSet> = self
            .sdk
            .mail_facade()
            .get_crypto_entity_client()
            .load_range(
                &mailbox.mailSets.mailSets,
                &GeneratedId::min_id(),
                100,
                ListLoadDirection::ASC,
            )
            .await
            .map_err(CoreError::from)?;
        Ok(FolderTree::new(folders))
    }

    /// Loads page `offset..offset+limit` of a folder, newest first.
    pub async fn load_folder_page(
        &self,
        folder: &FolderNode,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Mail>> {
        // The server caps a single load_range at 1000 entries. For larger
        // offsets we page through cursors. MailSetEntry ids sort by time, so
        // descending order is oldest-cursor-last.
        const SERVER_PAGE: usize = 1000;
        let client = self.sdk.mail_facade().get_crypto_entity_client();
        let entries_list = GeneratedId(folder.entries_list_id.clone());
        let mut entries: Vec<tuta_sdk::entities::generated::tutanota::MailSetEntry> = Vec::new();

        let mut remaining = offset + limit;
        let mut cursor = tuta_sdk::CustomId::default();
        while remaining > 0 {
            let want = remaining.min(SERVER_PAGE);
            let page: Vec<tuta_sdk::entities::generated::tutanota::MailSetEntry> = client
                .load_range(&entries_list, &cursor, want, ListLoadDirection::DESC)
                .await
                .map_err(CoreError::from)?;
            let got = page.len();
            if let Some(last) = page.last().and_then(|e| e._id.as_ref()) {
                cursor = last.element_id.clone();
            }
            entries.extend(page);
            if got < want {
                break;
            }
            remaining = remaining.saturating_sub(got);
        }
        let page: Vec<_> = entries.into_iter().skip(offset).take(limit).collect();

        // Mail ids carry their list id; batch per list to keep request count
        // low even when a folder mixes inbox and sent copies.
        let mut by_list: HashMap<String, Vec<GeneratedId>> = HashMap::new();
        for entry in &page {
            by_list
                .entry(entry.mail.list_id.0.clone())
                .or_default()
                .push(entry.mail.element_id.clone());
        }
        let mut mail_by_id: HashMap<(String, String), Mail> = HashMap::new();
        for (list, ids) in by_list {
            let list = GeneratedId(list);
            match client.load_multiple::<Mail>(&list, &ids).await {
                Ok(mails) => {
                    for mail in mails {
                        if let Some(id) = &mail._id {
                            mail_by_id
                                .insert((id.list_id.0.clone(), id.element_id.0.clone()), mail);
                        }
                    }
                }
                Err(error) => log::warn!("batch mail load failed for list {}: {error}", list.0),
            }
        }
        // Preserve folder order, not hash-map order.
        Ok(page
            .into_iter()
            .filter_map(|entry| {
                mail_by_id.remove(&(
                    entry.mail.list_id.0.clone(),
                    entry.mail.element_id.0.clone(),
                ))
            })
            .collect())
    }

    /// Loads a mail's body, using the blob archive for received mail and the
    /// draft list for drafts. Draft bodies that were just sent may 404; that
    /// is surfaced as `NotFound` so callers can refresh.
    pub async fn load_details(&self, mail: &Mail) -> Result<MailDetails> {
        let key = mail
            ._id
            .as_ref()
            .map(id_key)
            .unwrap_or_else(|| "unknown".to_string());
        if let Some(cached) = self.cache.lock().await.get(&key) {
            return Ok(cached);
        }
        let details = if let Some(id) = &mail.mailDetailsDraft {
            let draft: MailDetailsDraft = self.load_body(mail, BodySource::Draft(id)).await?;
            draft.details
        } else if let Some(id) = &mail.mailDetails {
            let blob: MailDetailsBlob = self.load_body(mail, BodySource::Blob(id)).await?;
            blob.details
        } else {
            return Err(CoreError::Protocol(
                "mail has neither received-details nor draft-details".into(),
            ));
        };
        self.cache.lock().await.put(key, details.clone());
        Ok(details)
    }

    async fn load_body<T: Entity + DeserializeOwned>(
        &self,
        mail: &Mail,
        source: BodySource<'_>,
    ) -> Result<T> {
        let type_ref = T::type_ref();
        let client = self.sdk.get_entity_client();
        let parsed = match source {
            BodySource::Blob(id) => {
                let body = self
                    .sdk
                    .blob_facade()
                    .load_blob_element(&type_ref, id)
                    .await
                    .map_err(CoreError::from)?;
                let raw_entities: Vec<_> = serde_json::from_slice(&body)
                    .map_err(|e| CoreError::Protocol(format!("invalid blob response: {e}")))?;
                let raw = raw_entities
                    .into_iter()
                    .next()
                    .ok_or_else(|| CoreError::Protocol("empty blob response".into()))?;
                client.parse_raw(&type_ref, raw).map_err(CoreError::from)?
            }
            BodySource::Draft(id) => client.load(&type_ref, id).await.map_err(CoreError::from)?,
        };
        let model = client
            .resolve_server_type_ref(&type_ref)
            .map_err(CoreError::from)?;
        let requirements = EntityDecryptionRequirements::for_entity(
            &model,
            &parsed,
            &self.sdk.type_model_provider,
        )
        .map_err(CoreError::from)?;
        let key = if requirements.session_key {
            Some(
                self.owner_key(
                    "Mail",
                    mail._ownerGroup.as_ref(),
                    mail._ownerEncSessionKey.as_ref(),
                    mail._ownerKeyVersion,
                )
                .await?,
            )
        } else {
            None
        };
        let decrypted = self
            .sdk
            .mail_facade()
            .get_crypto_entity_client()
            .decrypt_parsed(&type_ref, parsed, key)
            .await
            .map_err(CoreError::from)?;
        self.sdk
            .instance_mapper
            .parse_entity(decrypted)
            .map_err(|e| CoreError::Protocol(format!("failed to map {}: {e}", model.name)))
    }

    async fn owner_key(
        &self,
        owner: &str,
        owner_group: Option<&GeneratedId>,
        owner_enc_session_key: Option<&Vec<u8>>,
        owner_key_version: Option<i64>,
    ) -> Result<ResolvedSessionKey> {
        let owner_group = owner_group
            .ok_or_else(|| CoreError::Protocol(format!("{owner} missing _ownerGroup")))?;
        let encrypted = owner_enc_session_key
            .ok_or_else(|| CoreError::Protocol(format!("{owner} missing _ownerEncSessionKey")))?;
        let version = owner_key_version.unwrap_or(0).unsigned_abs();
        let client = self.sdk.mail_facade().get_crypto_entity_client();
        let group_key = client
            .get_crypto_facade()
            .get_key_loader_facade()
            .load_sym_group_key(owner_group, version, None)
            .await
            .map_err(|e| CoreError::Protocol(format!("failed to load group key: {e}")))?;
        let session_key = group_key
            .decrypt_aes_key(encrypted)
            .map_err(|e| CoreError::Protocol(format!("failed to decrypt session key: {e}")))?;
        Ok(ResolvedSessionKey {
            session_key,
            owner_enc_session_key: encrypted.clone(),
            owner_key_version: version,
            sender_identity_pub_key: None,
        })
    }

    /// Attachment metadata only; no payload is downloaded.
    pub async fn attachment_metas(
        &self,
        mail: &Mail,
    ) -> std::result::Result<Vec<AttachmentMeta>, AttachmentError> {
        let files = self.load_attachment_files(mail).await?;
        Ok(files
            .iter()
            .map(|(file, _)| AttachmentMeta::from_file(file))
            .collect())
    }

    /// Downloads and decrypts exactly one attachment. Resolves metadata again
    /// rather than retaining keys between UI actions.
    pub async fn download_attachment(
        &self,
        mail: &Mail,
        file_id: &str,
    ) -> std::result::Result<Vec<u8>, AttachmentError> {
        let files = self.load_attachment_files(mail).await?;
        let (file, key) = files
            .iter()
            .find(|(file, _)| {
                file._id
                    .as_ref()
                    .map(|id| id_key(id) == file_id)
                    .unwrap_or(false)
            })
            .ok_or(ContractError::NotInMailAttachments)?;
        self.load_file_attachment_data(file, &key.session_key)
            .await
            .map_err(classify)
    }

    async fn load_attachment_files(
        &self,
        mail: &Mail,
    ) -> std::result::Result<Vec<(TutanotaFile, ResolvedSessionKey)>, AttachmentError> {
        if mail.attachments.is_empty() {
            return Ok(Vec::new());
        }
        let type_ref = TutanotaFile::type_ref();
        let client = self.sdk.get_entity_client();
        let model = client
            .resolve_server_type_ref(&type_ref)
            .map_err(AttachmentError::Indeterminate)?;

        let mut lists: Vec<(GeneratedId, Vec<GeneratedId>)> = Vec::new();
        for id in &mail.attachments {
            match lists.iter_mut().find(|(list, _)| *list == id.list_id) {
                Some((_, elements)) if elements.contains(&id.element_id) => {
                    return Err(ContractError::DuplicateInBatch.into());
                }
                Some((_, elements)) => elements.push(id.element_id.clone()),
                None => lists.push((id.list_id.clone(), vec![id.element_id.clone()])),
            }
        }
        let mut parsed_files = HashMap::with_capacity(mail.attachments.len());
        for (list, elements) in &lists {
            let loaded = client
                .load_multiple(&type_ref, list, elements)
                .await
                .map_err(classify)?;
            for parsed in loaded {
                let id = list_element_id(&model, &parsed).ok_or_else(|| {
                    AttachmentError::Indeterminate(ApiCallError::internal(
                        "a loaded File has no id".to_string(),
                    ))
                })?;
                if id.list_id != *list {
                    return Err(ContractError::WrongList.into());
                }
                if !elements.contains(&id.element_id) {
                    return Err(ContractError::NotInMailAttachments.into());
                }
                if parsed_files.insert(pair(&id), parsed).is_some() {
                    return Err(ContractError::DuplicateInBatch.into());
                }
            }
        }
        if mail
            .attachments
            .iter()
            .any(|id| !parsed_files.contains_key(&pair(id)))
        {
            return Err(ContractError::IncompleteBatch.into());
        }

        let crypto_client = self.sdk.mail_facade().get_crypto_entity_client();
        let crypto = crypto_client.get_crypto_facade();
        let mut keys: HashMap<(GeneratedId, GeneratedId), ResolvedSessionKey> = HashMap::new();
        let mut from_bucket: Vec<IdTupleGenerated> = Vec::new();
        for id in &mail.attachments {
            let parsed = &parsed_files[&pair(id)];
            if has_owner_enc_session_key(&model, parsed) {
                let key = crypto
                    .resolve_session_key(parsed, &model)
                    .await
                    .map_err(|e| {
                        AttachmentError::Indeterminate(ApiCallError::internal(e.to_string()))
                    })?
                    .ok_or_else(|| {
                        AttachmentError::Indeterminate(ApiCallError::internal(
                            "File is encrypted but resolved no key".to_string(),
                        ))
                    })?;
                keys.insert(pair(id), key);
            } else {
                from_bucket.push(id.clone());
            }
        }
        if !from_bucket.is_empty() {
            let bucket = mail
                .bucketKey
                .as_ref()
                .ok_or(ContractError::KeyUnavailable)?;
            let mail_group = mail._ownerGroup.as_ref().ok_or_else(|| {
                AttachmentError::Indeterminate(ApiCallError::internal(
                    "Mail missing _ownerGroup".to_string(),
                ))
            })?;
            if from_bucket.iter().any(|id| {
                owner_group(&model, &parsed_files[&pair(id)]).as_ref() != Some(mail_group)
            }) {
                return Err(ContractError::UnsupportedGroupContext.into());
            }
            let resolved = crypto
                .resolve_bucket_instance_keys(bucket, mail_group, &from_bucket)
                .await
                .map_err(classify_bucket)?;
            keys.extend(from_bucket.iter().map(pair).zip(resolved));
        }

        let mut out = Vec::with_capacity(mail.attachments.len());
        for id in &mail.attachments {
            let parsed = parsed_files
                .remove(&pair(id))
                .expect("every attachment was loaded");
            let key = keys.remove(&pair(id)).expect("every attachment has a key");
            let decrypted = crypto_client
                .decrypt_parsed(&type_ref, parsed, Some(key.clone()))
                .await
                .map_err(classify)?;
            let mut file: TutanotaFile =
                self.sdk
                    .instance_mapper
                    .parse_entity(decrypted)
                    .map_err(|e| {
                        AttachmentError::Indeterminate(ApiCallError::internal(format!(
                            "failed to map File: {e}"
                        )))
                    })?;
            if !file._errors.is_empty() {
                return Err(AttachmentError::Indeterminate(ApiCallError::internal(
                    "File metadata did not decrypt".to_string(),
                )));
            }
            file._ownerKeyVersion = Some(i64::try_from(key.owner_key_version).map_err(|_| {
                AttachmentError::Indeterminate(ApiCallError::internal(
                    "File key version out of range".to_string(),
                ))
            })?);
            file._ownerEncSessionKey = Some(key.owner_enc_session_key.clone());
            out.push((file, key));
        }
        Ok(out)
    }

    async fn load_file_attachment_data(
        &self,
        file: &TutanotaFile,
        key: &GenericAesKey,
    ) -> std::result::Result<Vec<u8>, ApiCallError> {
        if file.blobs.is_empty() {
            return Ok(Vec::new());
        }
        let instance = file
            ._id
            .as_ref()
            .ok_or_else(|| ApiCallError::internal("File has no ID".to_string()))?;
        let mut archives: HashMap<GeneratedId, Vec<GeneratedId>> = HashMap::new();
        for blob in &file.blobs {
            archives
                .entry(blob.archiveId.clone())
                .or_default()
                .push(blob.blobId.clone());
        }
        let mut chunks = HashMap::new();
        for (archive, ids) in archives {
            let encrypted = self
                .sdk
                .blob_facade()
                .download_blobs(&archive, instance, ArchiveDataType::Attachments, &ids)
                .await?;
            for (id, bytes) in encrypted {
                let data = key.decrypt_data(&bytes).map_err(|e| {
                    ApiCallError::internal(format!("cannot decrypt attachment blob {id}: {e}"))
                })?;
                chunks.insert((archive.clone(), id), data);
            }
        }
        let mut result = Vec::new();
        for blob in &file.blobs {
            let data = chunks
                .get(&(blob.archiveId.clone(), blob.blobId.clone()))
                .ok_or_else(|| {
                    ApiCallError::internal(format!(
                        "server did not return blob {} of archive {}",
                        blob.blobId, blob.archiveId
                    ))
                })?;
            result.extend_from_slice(data);
        }
        Ok(result)
    }

    pub async fn move_mails(
        &self,
        mut mails: Vec<IdTupleGenerated>,
        target: IdTupleGenerated,
    ) -> Result<()> {
        mails.dedup();
        for chunk in mails.chunks(MAIL_BATCH_LIMIT) {
            self.sdk
                .get_service_executor()
                .post::<MoveMailService>(
                    MoveMailData {
                        _format: 0,
                        moveReason: None,
                        targetFolder: target.clone(),
                        mails: chunk.to_vec(),
                        excludeMailSet: None,
                    },
                    Default::default(),
                )
                .await
                .map_err(CoreError::from)?;
        }
        Ok(())
    }

    pub async fn delete_mails(
        &self,
        mails: Vec<IdTupleGenerated>,
        folder: IdTupleGenerated,
    ) -> Result<()> {
        let mut by_list: Vec<Vec<IdTupleGenerated>> = Vec::new();
        let mut index_by_list = HashMap::new();
        let mut seen = HashSet::new();
        for mail in mails {
            if !seen.insert(pair(&mail)) {
                continue;
            }
            let index = *index_by_list
                .entry(mail.list_id.clone())
                .or_insert_with(|| {
                    by_list.push(Vec::new());
                    by_list.len() - 1
                });
            by_list[index].push(mail);
        }
        for group in by_list {
            for chunk in group.chunks(MAIL_BATCH_LIMIT) {
                self.sdk
                    .get_service_executor()
                    .delete::<MailService>(
                        DeleteMailData {
                            _format: 0,
                            mails: chunk.to_vec(),
                            folder: Some(folder.clone()),
                        },
                        Default::default(),
                    )
                    .await
                    .map_err(CoreError::from)?;
            }
        }
        Ok(())
    }

    /// Decrypts an entity inlined in an event bus message. `None` means the
    /// payload did not carry enough key material and the caller should load
    /// the entity over REST.
    pub async fn decrypt_inline<T: Entity + DeserializeOwned>(
        &self,
        json: &str,
    ) -> Result<Option<T>> {
        let type_ref = T::type_ref();
        let client = self.sdk.get_entity_client();
        let model = client
            .resolve_server_type_ref(&type_ref)
            .map_err(CoreError::from)?;
        let raw = serde_json::from_str(json)
            .map_err(|e| CoreError::Protocol(format!("event JSON: {e}")))?;
        let parsed = client.parse_raw(&type_ref, raw).map_err(CoreError::from)?;
        let decrypted = if model.marked_encrypted() && has_kdf_nonce(&model, &parsed) {
            self.sdk
                .mail_facade()
                .get_crypto_entity_client()
                .decrypt_parsed(&type_ref, parsed, None)
                .await
                .map_err(CoreError::from)?
        } else if model.marked_encrypted() {
            let crypto_client = self.sdk.mail_facade().get_crypto_entity_client();
            let key = match crypto_client
                .get_crypto_facade()
                .resolve_session_key(&parsed, &model)
                .await
            {
                Ok(Some(key)) => key,
                Ok(None) => return Ok(None),
                Err(e) => {
                    log::debug!("inline key resolution failed for {}: {e}", model.name);
                    return Ok(None);
                }
            };
            self.decoder
                .decrypt_and_map(&model, parsed, key)
                .map_err(CoreError::from)?
        } else {
            parsed
        };
        self.sdk
            .instance_mapper
            .parse_entity(decrypted)
            .map(Some)
            .map_err(|e| CoreError::Protocol(format!("event map: {e}")))
    }
}
