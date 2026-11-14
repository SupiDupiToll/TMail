//! Live integration tests against a **dedicated test account**.
//!
//! These are ignored by default. Run them only with credentials for an account
//! you are willing to use destructively in tests:
//!
//! ```text
//! TMAIL_TEST_EMAIL=... TMAIL_TEST_PASSWORD=... \
//!   cargo test -p tmail-core --test live -- --ignored --nocapture
//! ```
//!
//! They never send, delete or move anything; they only read. Sending is
//! exercised manually via `docs/SMOKE.md`.

use tmail_core::{CoreService, LoginOutcome};

fn credentials() -> Option<(String, String)> {
    let email = std::env::var("TMAIL_TEST_EMAIL").ok()?;
    let password = std::env::var("TMAIL_TEST_PASSWORD").ok()?;
    if email.is_empty() || password.is_empty() {
        return None;
    }
    Some((email, password))
}

#[tokio::test]
#[ignore = "requires a dedicated Tuta test account"]
async fn sign_in_list_folders_and_open_the_newest_mail() {
    let Some((email, password)) = credentials() else {
        eprintln!("skipping: TMAIL_TEST_EMAIL/TMAIL_TEST_PASSWORD not set");
        return;
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let service = CoreService::new(dir.path().to_path_buf(), None).expect("core");

    match service.login(&email, &password).await.expect("login") {
        LoginOutcome::TotpRequired => {
            eprintln!("skipping: test account has TOTP enabled; exercise the UI path");
            return;
        }
        LoginOutcome::LoggedIn(snapshot) => {
            assert!(!snapshot.folders.is_empty(), "the account has no folders");
            let inbox = snapshot
                .folders
                .iter()
                .find(|folder| folder.kind == "inbox")
                .or_else(|| snapshot.folders.first())
                .expect("a folder");
            let mails = service
                .refresh_folder(&inbox.id, 20)
                .await
                .expect("folder load");
            if let Some(mail) = mails.first() {
                let detail = service.open_mail(&mail.id).await.expect("message open");
                assert!(!detail.summary.subject.is_empty());
                // Body may legitimately be empty; the important part is that
                // decryption and mapping completed without an error.
                let _ = detail.body_html.len();
            }
        }
    }

    service.logout().await;
}
