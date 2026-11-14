//! Serializable DTOs exchanged between the core, the Tauri shell and the UI.
//!
//! Nothing here carries key material or plaintext credentials. HTML is sent
//! as-is only after decryption; the renderer is responsible for sanitising it
//! inside a sandboxed iframe.

use serde::{Deserialize, Serialize};
use tuta_sdk::entities::generated::tutanota::{Mail, MailAddress, MailDetails, TutanotaFile};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Address {
    pub name: String,
    pub address: String,
}

impl From<&MailAddress> for Address {
    fn from(value: &MailAddress) -> Self {
        Self {
            name: value.name.clone(),
            address: value.address.clone(),
        }
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_empty() || self.name == self.address {
            f.write_str(&self.address)
        } else {
            write!(f, "{} <{}>", self.name, self.address)
        }
    }
}

/// List-row representation of a mail. Kept small: the UI virtualises large
/// folders and must not hold full bodies for every row.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MailSummary {
    pub id: String,
    pub element_id: String,
    pub list_id: String,
    pub subject: String,
    pub sender: Address,
    pub received_ms: i64,
    pub unread: bool,
    pub has_attachments: bool,
    pub attachment_count: usize,
    pub confidential: bool,
    pub is_draft: bool,
    pub folder_ids: Vec<String>,
    pub conversation_id: String,
    pub reply_type: i64,
    pub phishing_status: i64,
}

impl MailSummary {
    #[must_use]
    pub fn from_mail(mail: &Mail) -> Self {
        let (id, list_id, element_id) = mail._id.as_ref().map_or_else(
            || (String::new(), String::new(), String::new()),
            |id| {
                (
                    format!("{}/{}", id.list_id.0, id.element_id.0),
                    id.list_id.0.clone(),
                    id.element_id.0.clone(),
                )
            },
        );
        Self {
            id,
            element_id,
            list_id,
            subject: if mail.subject.is_empty() {
                "(no subject)".to_string()
            } else {
                mail.subject.clone()
            },
            sender: Address::from(&mail.sender),
            received_ms: i64::try_from(mail.receivedDate.as_millis()).unwrap_or(i64::MAX),
            unread: mail.unread,
            has_attachments: !mail.attachments.is_empty(),
            attachment_count: mail.attachments.len(),
            confidential: mail.confidential,
            is_draft: mail.mailDetailsDraft.is_some(),
            folder_ids: mail.sets.iter().map(|s| s.element_id.0.clone()).collect(),
            conversation_id: mail.conversationEntry.element_id.0.clone(),
            reply_type: mail.replyType,
            phishing_status: mail.phishingStatus,
        }
    }

    /// Plain-text haystack for local search and screen-reader previews.
    #[must_use]
    pub fn search_text(&self) -> String {
        format!(
            "{} {} {}",
            self.subject, self.sender.name, self.sender.address
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttachmentMeta {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub is_inline: bool,
}

impl AttachmentMeta {
    #[must_use]
    pub fn from_file(file: &TutanotaFile) -> Self {
        let id = file
            ._id
            .clone()
            .map(|id| format!("{}/{}", id.list_id.0, id.element_id.0))
            .unwrap_or_default();
        Self {
            id,
            name: file.name.clone(),
            mime_type: file
                .mimeType
                .clone()
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            size: file.size,
            is_inline: file.cid.is_some(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Recipients {
    pub to: Vec<Address>,
    pub cc: Vec<Address>,
    pub bcc: Vec<Address>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MailDetail {
    pub summary: MailSummary,
    pub recipients: Recipients,
    pub reply_to: Vec<Address>,
    pub sent_ms: i64,
    /// Decrypted body HTML. Must be sanitised by the renderer.
    pub body_html: String,
    /// Derived plain text used for local FTS and the reading fallback.
    pub body_text: String,
    pub attachments: Vec<AttachmentMeta>,
    /// Raw details kept in the core only for forwarding/replying; not serialised.
    #[serde(skip)]
    pub raw: Option<MailDetails>,
}

impl MailDetail {
    #[must_use]
    pub fn build(mail: &Mail, details: &MailDetails, attachments: Vec<AttachmentMeta>) -> Self {
        let html = details
            .body
            .text
            .clone()
            .or_else(|| details.body.compressedText.clone())
            .unwrap_or_default();
        let text = html_to_text(&html);
        Self {
            summary: MailSummary::from_mail(mail),
            recipients: Recipients {
                to: details
                    .recipients
                    .toRecipients
                    .iter()
                    .map(Address::from)
                    .collect(),
                cc: details
                    .recipients
                    .ccRecipients
                    .iter()
                    .map(Address::from)
                    .collect(),
                bcc: details
                    .recipients
                    .bccRecipients
                    .iter()
                    .map(Address::from)
                    .collect(),
            },
            reply_to: details
                .replyTos
                .iter()
                .map(|a| Address {
                    name: a.name.clone(),
                    address: a.address.clone(),
                })
                .collect(),
            sent_ms: i64::try_from(details.sentDate.as_millis()).unwrap_or(0),
            body_html: html,
            body_text: text,
            attachments,
            raw: Some(details.clone()),
        }
    }
}

/// Draft input from the composer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DraftInput {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_html: String,
    /// Send as text/plain (Tuta's `plaintext` flag).
    #[serde(default)]
    pub plaintext: bool,
    #[serde(default)]
    pub confidential: bool,
    /// Reply/forward linkage, if any.
    #[serde(default)]
    pub previous_mail: Option<String>,
    #[serde(default)]
    pub previous_message_id: Option<String>,
    #[serde(default)]
    pub conversation_type: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendReceipt {
    pub message_id: String,
    pub draft_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub mail: MailSummary,
    pub snippet: String,
}

/// Minimal HTML-to-text for FTS, previews and quote generation. This is not a
/// sanitiser: rendering always goes through the UI's strict pipeline.
#[must_use]
pub fn html_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut tag = String::new();
    for ch in html.chars() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
                let name = tag.trim_start_matches('/').to_ascii_lowercase();
                if name == "script" || name == "style" {
                    in_script = !tag.starts_with('/');
                }
                if matches!(
                    name.as_str(),
                    "br" | "p" | "div" | "li" | "tr" | "h1" | "h2" | "h3"
                ) {
                    out.push('\n');
                }
                tag.clear();
            } else {
                tag.push(ch);
            }
        } else if ch == '<' {
            in_tag = true;
        } else if !in_script {
            out.push(ch);
        }
    }
    decode_entities(&out)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_display_falls_back_to_the_mailbox() {
        let a = Address {
            name: String::new(),
            address: "a@b.c".into(),
        };
        assert_eq!(a.to_string(), "a@b.c");
        let a = Address {
            name: "Ada".into(),
            address: "a@b.c".into(),
        };
        assert_eq!(a.to_string(), "Ada <a@b.c>");
    }

    #[test]
    fn html_to_text_drops_tags_and_scripts() {
        let text = html_to_text("<p>Hello <b>world</b></p><script>bad()</script><div>Next</div>");
        assert!(text.contains("Hello world"));
        assert!(text.contains("Next"));
        assert!(!text.contains("bad"));
        assert!(!text.contains('<'));
    }
}
