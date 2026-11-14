//! `MailSetEntry` id codec.
//!
//! A mail's placement in a folder is a `MailSetEntry` whose element id packs
//! a truncated timestamp and the mail id. Knowing the codec lets realtime
//! events identify a mail without an extra REST round-trip.
//!
//! Adapted from `spartanz51/tutabridge` (`crates/tuta/src/mail_set_entry_id.rs`,
//! GPL-3.0), with tests and API comments retained/modified. See `NOTICE.md`.

use base64::prelude::{Engine, BASE64_URL_SAFE_NO_PAD};
use thiserror::Error;
use tuta_sdk::date::DateTime;
use tuta_sdk::util::BASE64_EXT;
use tuta_sdk::{CustomId, GeneratedId};

const MAIL_ID_BYTES: usize = 9;
const TIMESTAMP_BYTES: usize = 4;
const ID_BYTES: usize = TIMESTAMP_BYTES + MAIL_ID_BYTES;
const TIMESTAMP_SHIFT: u32 = 10;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MailSetEntryIdError {
    #[error("invalid MailSetEntry id encoding")]
    InvalidEncoding,
    #[error("unexpected MailSetEntry id length: {0} bytes")]
    WrongLength(usize),
    #[error("invalid mail id inside MailSetEntry")]
    InvalidMailId,
}

/// Builds the `MailSetEntry` element id for `mail_id` received at `date`.
///
/// Returns an error instead of panicking on malformed ids, unlike the
/// reference implementation: event payloads are untrusted input.
pub fn construct(date: DateTime, mail_id: &GeneratedId) -> Result<CustomId, MailSetEntryIdError> {
    let mail_bytes = BASE64_EXT
        .decode(mail_id.as_str())
        .map_err(|_| MailSetEntryIdError::InvalidMailId)?;
    if mail_bytes.len() != MAIL_ID_BYTES {
        return Err(MailSetEntryIdError::InvalidMailId);
    }
    let mut buffer = [0u8; ID_BYTES];
    buffer[..TIMESTAMP_BYTES]
        .copy_from_slice(&((date.as_millis() >> TIMESTAMP_SHIFT) as u32).to_be_bytes());
    buffer[TIMESTAMP_BYTES..].copy_from_slice(&mail_bytes);
    Ok(CustomId(BASE64_URL_SAFE_NO_PAD.encode(buffer)))
}

/// Splits a `MailSetEntry` id into its (quantised) date and mail id.
pub fn deconstruct(id: &CustomId) -> Result<(DateTime, GeneratedId), MailSetEntryIdError> {
    let buffer = BASE64_URL_SAFE_NO_PAD
        .decode(id.as_str())
        .map_err(|_| MailSetEntryIdError::InvalidEncoding)?;
    if buffer.len() != ID_BYTES {
        return Err(MailSetEntryIdError::WrongLength(buffer.len()));
    }
    let mut ts = [0u8; TIMESTAMP_BYTES];
    ts.copy_from_slice(&buffer[..TIMESTAMP_BYTES]);
    let millis = u64::from(u32::from_be_bytes(ts)) << TIMESTAMP_SHIFT;
    Ok((
        DateTime::from_millis(millis),
        GeneratedId(BASE64_EXT.encode(&buffer[TIMESTAMP_BYTES..])),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_official_test_vector() {
        let mail = GeneratedId("-----------0".into());
        let id = construct(DateTime::from_millis(1_507_038_373_000), &mail).unwrap();
        assert_eq!(id.as_str(), "V7ifKQAAAAAAAAAAAQ");
        assert_eq!(deconstruct(&id).unwrap().1.as_str(), mail.as_str());
    }

    #[test]
    fn rejects_malformed_ids() {
        assert_eq!(
            deconstruct(&CustomId("AAAA".into())).unwrap_err(),
            MailSetEntryIdError::WrongLength(3)
        );
        assert!(construct(
            DateTime::from_millis(0),
            &GeneratedId("not-a-generated-id".into())
        )
        .is_err());
    }
}
