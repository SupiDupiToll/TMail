//! Compose, draft creation and sending.
//!
//! Adapted from `spartanz51/tutabridge` (`crates/bridge/src/tuta.rs`,
//! GPL-3.0), reworked around a typed compose model rather than SMTP parsing.
//! Crypto and the draft/send services stay in the official SDK. See
//! `NOTICE.md`.

use std::sync::Arc;

use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use crypto_primitives::aes::{Aes256Key, InitializationVector};
use crypto_primitives::key::GenericAesKey;
use crypto_primitives::randomizer_facade::RandomizerFacade;
use tuta_sdk::blobs::blob_facade::FileData;
use tuta_sdk::entities::generated::tutanota::{
    AttachmentKeyData, DraftAttachment, DraftCreateData, DraftData, DraftRecipient,
    InternalRecipientKeyData, NewDraftAttachment, SendDraftData, SendDraftParameters,
};
use tuta_sdk::services::generated::tutanota::{DraftService, SendDraftService};
use tuta_sdk::services::ExtraServiceParams;
use tuta_sdk::tutanota_constants::ArchiveDataType;
use tuta_sdk::{ApiCallError, CustomId, GeneratedId, IdTupleGenerated, LoggedInSdk};

use crate::error::{CoreError, Result};

/// The server's `ConversationType` values.
pub const CONVERSATION_NEW: i64 = 0;
pub const CONVERSATION_REPLY: i64 = 1;
pub const CONVERSATION_FORWARD: i64 = 2;

#[derive(Clone, Debug)]
pub struct OutgoingAttachment {
    pub filename: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Outgoing {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_html: String,
    pub plaintext: bool,
    pub confidential: bool,
    /// Password for external recipients when confidential mode is enabled.
    pub confidential_password: Option<String>,
    pub sender_name: String,
    pub sender_address: String,
    pub previous_message_id: Option<String>,
    pub conversation_type: i64,
    pub attachments: Vec<OutgoingAttachment>,
}

struct CreatedDraft {
    id: IdTupleGenerated,
    session_key: GenericAesKey,
    attachment_keys: Vec<GenericAesKey>,
}

fn is_tuta_address(address: &str) -> bool {
    let address = address.trim().to_ascii_lowercase();
    ["@tuta.com", "@tutanota.com", "@tuta.io", "@tutanota.de"]
        .iter()
        .any(|domain| address.ends_with(domain))
}

fn all_tuta(outgoing: &Outgoing) -> bool {
    let addresses = outgoing.to.iter().chain(&outgoing.cc).chain(&outgoing.bcc);
    let mut any = false;
    for address in addresses {
        any = true;
        if !is_tuta_address(address) {
            return false;
        }
    }
    any
}

fn effective_confidential(outgoing: &Outgoing) -> bool {
    outgoing.confidential || all_tuta(outgoing)
}

/// Creates a draft and immediately submits it for sending.
pub async fn send(sdk: &Arc<LoggedInSdk>, outgoing: &Outgoing) -> Result<(String, String)> {
    validate(outgoing)?;
    let randomizer = RandomizerFacade::from_core(rand_core::OsRng);
    let confidential = effective_confidential(outgoing);
    let draft = create_draft(sdk, outgoing, confidential, &randomizer).await?;

    let attachment_keys = if draft.attachment_keys.is_empty() {
        Vec::new()
    } else {
        attachment_key_data(sdk, &draft.id, &draft.attachment_keys, &randomizer).await?
    };
    let (mail_session_key, bucket_enc_mail_session_key, internal_keys, auth_status) =
        if confidential {
            let bucket_key: GenericAesKey = Aes256Key::generate(&randomizer).into();
            let bucket_enc = bucket_key.encrypt_key(
                &draft.session_key,
                InitializationVector::generate(&randomizer),
            );
            let crypto_client = sdk.mail_facade().get_crypto_entity_client();
            let crypto = crypto_client.get_crypto_facade();
            let sender_group_id = sdk.get_user_group_id();
            let mut keys = Vec::new();
            let mut all_tuta_crypt = true;
            for address in outgoing.to.iter().chain(&outgoing.cc).chain(&outgoing.bcc) {
                let key = crypto
                    .encrypt_bucket_key_for_recipient(&bucket_key, address, &sender_group_id)
                    .await
                    .map_err(|error| {
                        CoreError::Protocol(format!("recipient key encryption failed: {error}"))
                    })?;
                all_tuta_crypt &= key.protocol_version() == 2;
                keys.push(InternalRecipientKeyData {
                    _id: None,
                    mailAddress: address.trim().to_ascii_lowercase(),
                    pubEncBucketKey: key.bytes().to_vec(),
                    recipientKeyVersion: key.recipient_key_version(),
                    protocolVersion: key.protocol_version(),
                    senderKeyVersion: key.sender_key_version(),
                });
            }
            let auth_status = if all_tuta_crypt {
                draft
                    .session_key
                    .encrypt_data(b"4", InitializationVector::generate(&randomizer))
                    .ok()
            } else {
                None
            };
            (None, Some(bucket_enc), keys, auth_status)
        } else {
            (
                Some(draft.session_key.as_bytes().to_vec()),
                None,
                Vec::new(),
                None,
            )
        };
    let parameters_id =
        CustomId(BASE64_URL_SAFE_NO_PAD.encode(randomizer.generate_random_array::<4>()));
    let data = build_send_data(
        mail_session_key,
        bucket_enc_mail_session_key,
        internal_keys,
        auth_status,
        draft.id.clone(),
        parameters_id,
        attachment_keys,
        outgoing.plaintext,
    );
    let sent = sdk
        .get_service_executor()
        .post::<SendDraftService>(data, ExtraServiceParams::default())
        .await
        .map_err(CoreError::from)?;
    Ok((
        sent.messageId,
        format!("{}/{}", draft.id.list_id.0, draft.id.element_id.0),
    ))
}

/// Saves a draft without sending it and returns the draft mail id.
pub async fn save_draft(sdk: &Arc<LoggedInSdk>, outgoing: &Outgoing) -> Result<String> {
    validate(outgoing)?;
    let randomizer = RandomizerFacade::from_core(rand_core::OsRng);
    let draft = create_draft(sdk, outgoing, effective_confidential(outgoing), &randomizer).await?;
    Ok(format!("{}/{}", draft.id.list_id.0, draft.id.element_id.0))
}

fn validate(outgoing: &Outgoing) -> Result<()> {
    if outgoing.to.is_empty() && outgoing.cc.is_empty() && outgoing.bcc.is_empty() {
        return Err(CoreError::InvalidInput("add at least one recipient".into()));
    }
    if outgoing
        .to
        .iter()
        .chain(&outgoing.cc)
        .chain(&outgoing.bcc)
        .any(|a| {
            let a = a.trim();
            a.is_empty() || !a.contains('@') || a.contains(char::is_whitespace)
        })
    {
        return Err(CoreError::InvalidInput(
            "check the recipient addresses".into(),
        ));
    }
    if outgoing.confidential && !all_tuta(outgoing) {
        // The external-group/Bcrypt flow is not exposed by the SDK yet. Refuse
        // before a draft is created so nothing is left behind on the server.
        return Err(CoreError::InvalidInput(
            "password-protected mail to external recipients is not available yet".into(),
        ));
    }
    Ok(())
}

async fn create_draft(
    sdk: &Arc<LoggedInSdk>,
    outgoing: &Outgoing,
    confidential: bool,
    randomizer: &RandomizerFacade,
) -> Result<CreatedDraft> {
    let session_key: GenericAesKey = Aes256Key::generate(randomizer).into();
    let mail_group_id = sdk
        .mail_facade()
        .get_group_id_for_mail_address(&outgoing.sender_address)
        .await
        .map_err(CoreError::from)?;
    let group_key = sdk
        .get_current_sym_group_key(&mail_group_id)
        .await
        .map_err(CoreError::from)?;
    let owner_enc_session_key = group_key
        .object
        .encrypt_key(&session_key, InitializationVector::generate(randomizer));
    let owner_key_version = i64::try_from(group_key.version)
        .map_err(|_| CoreError::Protocol("group key version out of range".into()))?;

    let (added_attachments, attachment_keys) = build_attachments(
        sdk,
        &outgoing.attachments,
        &mail_group_id,
        &group_key.object,
        group_key.version,
        randomizer,
    )
    .await?;

    let data = DraftData {
        _id: None,
        subject: outgoing.subject.clone(),
        bodyText: outgoing.body_html.clone(),
        senderMailAddress: outgoing.sender_address.clone(),
        senderName: if outgoing.sender_name.is_empty() {
            outgoing.sender_address.clone()
        } else {
            outgoing.sender_name.clone()
        },
        confidential,
        method: 0,
        compressedBodyText: Some(outgoing.body_html.clone()),
        toRecipients: recipients(&outgoing.to),
        ccRecipients: recipients(&outgoing.cc),
        bccRecipients: recipients(&outgoing.bcc),
        addedAttachments: added_attachments,
        removedAttachments: Vec::new(),
        // Tuta drops Reply-To on outgoing mail; see the reference analysis.
        replyTos: Vec::new(),
        _errors: Default::default(),
    };
    let create = DraftCreateData {
        _format: 0,
        previousMessageId: outgoing.previous_message_id.clone(),
        conversationType: outgoing.conversation_type,
        ownerEncSessionKey: owner_enc_session_key,
        ownerKeyVersion: owner_key_version,
        draftData: data,
        _errors: Default::default(),
    };
    let created = sdk
        .get_service_executor()
        .post::<DraftService>(
            create,
            ExtraServiceParams {
                session_key: Some(session_key.clone()),
                ..Default::default()
            },
        )
        .await
        .map_err(CoreError::from)?;
    Ok(CreatedDraft {
        id: created.draft,
        session_key,
        attachment_keys,
    })
}

fn recipients(addresses: &[String]) -> Vec<DraftRecipient> {
    addresses
        .iter()
        .map(|address| {
            let address = address.trim();
            DraftRecipient {
                _id: None,
                name: address.to_string(),
                mailAddress: address.to_string(),
                _errors: Default::default(),
            }
        })
        .collect()
}

async fn build_attachments(
    sdk: &Arc<LoggedInSdk>,
    attachments: &[OutgoingAttachment],
    mail_group_id: &GeneratedId,
    mail_group_key: &GenericAesKey,
    mail_group_key_version: u64,
    randomizer: &RandomizerFacade,
) -> Result<(Vec<DraftAttachment>, Vec<GenericAesKey>)> {
    if attachments.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let session_keys: Vec<GenericAesKey> = (0..attachments.len())
        .map(|_| Aes256Key::generate(randomizer).into())
        .collect();
    let files: Vec<FileData> = attachments
        .iter()
        .zip(&session_keys)
        .map(|(attachment, key)| FileData {
            session_key: key.clone(),
            data: attachment.data.as_slice(),
        })
        .collect();
    let tokens = sdk
        .blob_facade()
        .encrypt_and_upload_multiple(ArchiveDataType::Attachments, mail_group_id, files.iter())
        .await
        .map_err(CoreError::from)?;
    if tokens.len() != attachments.len() {
        return Err(CoreError::Protocol(
            "attachment upload count mismatch".into(),
        ));
    }
    let mut drafts = Vec::with_capacity(attachments.len());
    for ((attachment, key), token) in attachments.iter().zip(&session_keys).zip(tokens) {
        let enc_name = key
            .encrypt_data(
                attachment.filename.as_bytes(),
                InitializationVector::generate(randomizer),
            )
            .map_err(|e| CoreError::Protocol(format!("encrypt attachment name: {e}")))?;
        let enc_mime = key
            .encrypt_data(
                attachment.mime_type.as_bytes(),
                InitializationVector::generate(randomizer),
            )
            .map_err(|e| CoreError::Protocol(format!("encrypt attachment type: {e}")))?;
        let owner_enc_key =
            mail_group_key.encrypt_key(key, InitializationVector::generate(randomizer));
        drafts.push(DraftAttachment {
            _id: Some(random_custom_id(randomizer)),
            ownerEncFileSessionKey: owner_enc_key,
            ownerKeyVersion: i64::try_from(mail_group_key_version).unwrap_or(0),
            newFile: Some(NewDraftAttachment {
                _id: Some(random_custom_id(randomizer)),
                encFileName: enc_name,
                encMimeType: enc_mime,
                encCid: None,
                referenceTokens: token,
            }),
            existingFile: None,
        });
    }
    Ok((drafts, session_keys))
}

async fn attachment_key_data(
    sdk: &Arc<LoggedInSdk>,
    draft_id: &IdTupleGenerated,
    keys: &[GenericAesKey],
    randomizer: &RandomizerFacade,
) -> Result<Vec<AttachmentKeyData>> {
    let mail: tuta_sdk::entities::generated::tutanota::Mail = sdk
        .mail_facade()
        .get_crypto_entity_client()
        .load(draft_id)
        .await
        .map_err(CoreError::from)?;
    if mail.attachments.len() != keys.len() {
        return Err(CoreError::Protocol(
            "server attachment count mismatch".into(),
        ));
    }
    Ok(mail
        .attachments
        .into_iter()
        .zip(keys)
        .map(|(file, key)| AttachmentKeyData {
            _id: Some(random_custom_id(randomizer)),
            bucketEncFileSessionKey: None,
            fileSessionKey: Some(key.as_bytes().to_vec()),
            file,
        })
        .collect())
}

fn build_send_data(
    mail_session_key: Option<Vec<u8>>,
    bucket_enc_mail_session_key: Option<Vec<u8>>,
    internal_recipient_key_data: Vec<InternalRecipientKeyData>,
    session_enc_encryption_auth_status: Option<Vec<u8>>,
    draft: IdTupleGenerated,
    parameters_id: CustomId,
    attachment_key_data: Vec<AttachmentKeyData>,
    plaintext: bool,
) -> SendDraftData {
    let parameters = SendDraftParameters {
        _id: Some(parameters_id),
        language: "en".into(),
        mailSessionKey: mail_session_key.clone(),
        bucketEncMailSessionKey: bucket_enc_mail_session_key.clone(),
        senderNameUnencrypted: None,
        plaintext,
        calendarMethod: false,
        sessionEncEncryptionAuthStatus: session_enc_encryption_auth_status.clone(),
        mail: draft.clone(),
        internalRecipientKeyData: internal_recipient_key_data.clone(),
        secureExternalRecipientKeyData: Vec::new(),
        symEncInternalRecipientKeyData: Vec::new(),
        attachmentKeyData: attachment_key_data.clone(),
    };
    SendDraftData {
        _format: 0,
        language: "en".into(),
        mailSessionKey: mail_session_key,
        bucketEncMailSessionKey: bucket_enc_mail_session_key,
        senderNameUnencrypted: None,
        plaintext,
        calendarMethod: false,
        sessionEncEncryptionAuthStatus: session_enc_encryption_auth_status,
        sendAt: None,
        allowUndo: false,
        internalRecipientKeyData: internal_recipient_key_data,
        secureExternalRecipientKeyData: Vec::new(),
        attachmentKeyData: attachment_key_data,
        mail: draft.clone(),
        symEncInternalRecipientKeyData: Vec::new(),
        parameters: Some(parameters),
    }
}

fn random_custom_id(randomizer: &RandomizerFacade) -> CustomId {
    CustomId(BASE64_URL_SAFE_NO_PAD.encode(randomizer.generate_random_array::<4>()))
}

#[allow(dead_code)]
fn _api(error: ApiCallError) -> CoreError {
    CoreError::from(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outgoing() -> Outgoing {
        Outgoing {
            to: vec!["a@example.com".into()],
            cc: vec![],
            bcc: vec![],
            subject: "Hi".into(),
            body_html: "<p>Hello</p>".into(),
            plaintext: false,
            confidential: false,
            confidential_password: None,
            sender_name: "Me".into(),
            sender_address: "me@tuta.com".into(),
            previous_message_id: None,
            conversation_type: CONVERSATION_NEW,
            attachments: vec![],
        }
    }

    #[test]
    fn rejects_empty_and_malformed_recipients() {
        let mut mail = outgoing();
        mail.to.clear();
        assert!(validate(&mail).is_err());
        mail.to = vec!["not an address".into()];
        assert!(validate(&mail).is_err());
        mail.to = vec!["a@example.com".into()];
        assert!(validate(&mail).is_ok());
    }

    #[test]
    fn external_confidential_is_refused_before_creating_a_draft() {
        let mut mail = outgoing();
        mail.confidential = true;
        assert!(validate(&mail).is_err());
        mail.confidential_password = Some("correct horse battery staple".into());
        assert!(validate(&mail).is_err());
        mail.to = vec!["friend@tuta.com".into()];
        assert!(validate(&mail).is_ok());
    }

    #[test]
    fn tuta_recipients_are_always_confidential() {
        let mut mail = outgoing();
        mail.to = vec!["friend@tuta.com".into()];
        assert!(effective_confidential(&mail));
        mail.to = vec!["friend@example.com".into()];
        assert!(!effective_confidential(&mail));
        mail.confidential = true;
        assert!(effective_confidential(&mail));
    }
}
