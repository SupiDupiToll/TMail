use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use tmail_core::models::{DraftInput, MailDetail, MailSummary, SearchHit};
use tmail_core::service::{IncomingAttachment, Snapshot};
use tmail_core::{CoreError, CoreService, LoginOutcome};

pub struct AppState {
    pub service: Arc<CoreService>,
    pub quitting: AtomicBool,
    pub tray_available: AtomicBool,
    pub pending_mailto: std::sync::Mutex<Option<crate::mailto::MailtoRequest>>,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    code: String,
    message: String,
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        Self {
            code: error.code().to_string(),
            message: error.to_string(),
        }
    }
}

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        Self {
            code: "internal".into(),
            message,
        }
    }
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Serialize)]
pub struct AppInfo {
    version: String,
    product: String,
    api_url: String,
    mock: bool,
}

fn notification_icon(app: &AppHandle) -> Option<String> {
    let dir = app.path().app_cache_dir().ok()?;
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("notification-icon.png");
    const ICON: &[u8] = include_bytes!("../icons/128x128.png");
    if std::fs::read(&path).ok().as_deref() != Some(ICON) {
        std::fs::write(&path, ICON).ok()?;
    }
    Some(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn send_notification(
    app: AppHandle,
    state: State<'_, AppState>,
    title: String,
    body: String,
) -> ApiResult<()> {
    let _ = &state;
    let mut builder = app.notification().builder().title(title).body(body);
    if let Some(icon) = notification_icon(&app) {
        builder = builder.icon(icon);
    }
    builder
        .show()
        .map_err(|error| ApiError::from(format!("notification failed: {error}")))
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: tmail_core::VERSION.to_string(),
        product: tmail_core::PRODUCT_NAME.to_string(),
        api_url: state.service.api_url().to_string(),
        mock: false,
    }
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LoginResponse {
    LoggedIn { snapshot: Snapshot },
    TotpRequired,
}

impl From<LoginOutcome> for LoginResponse {
    fn from(value: LoginOutcome) -> Self {
        match value {
            LoginOutcome::LoggedIn(snapshot) => Self::LoggedIn { snapshot },
            LoginOutcome::TotpRequired => Self::TotpRequired,
        }
    }
}

#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> ApiResult<LoginResponse> {
    state
        .service
        .login(&email, &password)
        .await
        .map(LoginResponse::from)
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn submit_totp(state: State<'_, AppState>, code: u32) -> ApiResult<Snapshot> {
    state
        .service
        .submit_totp(code)
        .await
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn cancel_login(state: State<'_, AppState>) -> ApiResult<()> {
    state.service.cancel_login().await;
    Ok(())
}

#[derive(Serialize)]
pub struct LastAccount {
    email: String,
    api_url: String,
}

#[tauri::command]
pub fn last_account(state: State<'_, AppState>) -> Option<LastAccount> {
    state
        .service
        .last_account()
        .map(|(email, api_url)| LastAccount { email, api_url })
}

#[tauri::command]
pub async fn resume(state: State<'_, AppState>, email: String) -> ApiResult<Option<Snapshot>> {
    state.service.resume(&email).await.map_err(ApiError::from)
}

#[tauri::command]
pub async fn snapshot(state: State<'_, AppState>) -> ApiResult<Snapshot> {
    state.service.snapshot().await.map_err(ApiError::from)
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> ApiResult<()> {
    state.service.logout().await;
    Ok(())
}

#[tauri::command]
pub async fn classify_recipients(
    state: State<'_, AppState>,
    addresses: Vec<String>,
) -> ApiResult<Vec<tmail_core::service::RecipientKind>> {
    state
        .service
        .classify_recipients(addresses)
        .await
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn list_folders(
    state: State<'_, AppState>,
) -> ApiResult<Vec<tmail_core::folders::FolderNode>> {
    state.service.folder_tree().map_err(ApiError::from)
}

#[tauri::command]
pub async fn unread_counts(
    state: State<'_, AppState>,
) -> ApiResult<Vec<tmail_core::service::FolderUnread>> {
    state.service.unread_counts().map_err(ApiError::from)
}

#[tauri::command]
pub async fn open_folder(
    state: State<'_, AppState>,
    folder_id: String,
    limit: Option<usize>,
) -> ApiResult<Vec<MailSummary>> {
    state
        .service
        .open_folder(&folder_id, limit.unwrap_or(200))
        .await
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn refresh_folder(
    state: State<'_, AppState>,
    folder_id: String,
    limit: Option<usize>,
) -> ApiResult<Vec<MailSummary>> {
    state
        .service
        .refresh_folder(&folder_id, limit.unwrap_or(200))
        .await
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn open_mail(state: State<'_, AppState>, id: String) -> ApiResult<MailDetail> {
    state.service.open_mail(&id).await.map_err(ApiError::from)
}

#[tauri::command]
pub async fn search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
) -> ApiResult<Vec<SearchHit>> {
    state
        .service
        .search(&query, limit.unwrap_or(50))
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn set_unread(
    state: State<'_, AppState>,
    ids: Vec<String>,
    unread: bool,
) -> ApiResult<()> {
    state
        .service
        .set_unread(&ids, unread)
        .await
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn move_mails(
    state: State<'_, AppState>,
    ids: Vec<String>,
    target_folder: String,
) -> ApiResult<()> {
    state
        .service
        .move_mails(&ids, &target_folder)
        .await
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn trash(state: State<'_, AppState>, ids: Vec<String>) -> ApiResult<()> {
    state.service.trash(&ids).await.map_err(ApiError::from)
}

#[tauri::command]
pub async fn archive(state: State<'_, AppState>, ids: Vec<String>) -> ApiResult<()> {
    state.service.archive(&ids).await.map_err(ApiError::from)
}

#[tauri::command]
pub async fn delete_forever(
    state: State<'_, AppState>,
    ids: Vec<String>,
    folder_id: String,
) -> ApiResult<()> {
    state
        .service
        .delete_forever(&ids, &folder_id)
        .await
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn set_current_folder(
    state: State<'_, AppState>,
    folder_id: Option<String>,
) -> ApiResult<()> {
    state.service.set_current_folder(folder_id);
    Ok(())
}

#[tauri::command]
pub async fn clear_cache(state: State<'_, AppState>) -> ApiResult<()> {
    state.service.clear_cache().map_err(ApiError::from)
}

#[derive(Clone, Debug, Deserialize)]
pub struct UploadAttachment {
    pub filename: String,
    pub mime_type: String,
    /// Standard base64; the UI encodes the bytes it read from a File.
    pub data: String,
}

impl UploadAttachment {
    fn decode(self) -> ApiResult<IncomingAttachment> {
        use base64::prelude::{Engine, BASE64_STANDARD};
        let data = BASE64_STANDARD
            .decode(self.data.as_bytes())
            .map_err(|_| ApiError::from("attachment data is not valid base64".to_string()))?;
        Ok(IncomingAttachment {
            filename: self.filename,
            mime_type: self.mime_type,
            data,
        })
    }
}

#[tauri::command]
pub async fn send_mail(
    state: State<'_, AppState>,
    draft: DraftInput,
    attachments: Vec<UploadAttachment>,
) -> ApiResult<tmail_core::models::SendReceipt> {
    let attachments = attachments
        .into_iter()
        .map(UploadAttachment::decode)
        .collect::<ApiResult<Vec<_>>>()?;
    state
        .service
        .send(draft, attachments)
        .await
        .map_err(ApiError::from)
}

#[tauri::command]
pub async fn save_draft(
    state: State<'_, AppState>,
    draft: DraftInput,
    attachments: Vec<UploadAttachment>,
) -> ApiResult<String> {
    let attachments = attachments
        .into_iter()
        .map(UploadAttachment::decode)
        .collect::<ApiResult<Vec<_>>>()?;
    state
        .service
        .save_draft(draft, attachments)
        .await
        .map_err(ApiError::from)
}

#[derive(Serialize)]
pub struct AttachmentData {
    name: String,
    mime_type: String,
    /// Standard base64 of the decrypted attachment bytes.
    data: String,
}

#[tauri::command]
pub async fn read_attachment(
    state: State<'_, AppState>,
    mail_id: String,
    attachment_id: String,
    name: String,
    mime_type: String,
) -> ApiResult<AttachmentData> {
    let bytes = state
        .service
        .download_attachment(&mail_id, &attachment_id)
        .await?;
    if bytes.len() > 12 * 1024 * 1024 {
        return Err(ApiError::from(
            "this attachment is too large to preview; save it instead".to_string(),
        ));
    }
    use base64::prelude::{Engine, BASE64_STANDARD};
    Ok(AttachmentData {
        name,
        mime_type,
        data: BASE64_STANDARD.encode(bytes),
    })
}

#[derive(Serialize)]
pub struct SaveAttachmentResult {
    saved: bool,
    path: Option<String>,
}

#[tauri::command]
pub async fn save_attachment(
    app: AppHandle,
    state: State<'_, AppState>,
    mail_id: String,
    attachment_id: String,
    name: String,
) -> ApiResult<SaveAttachmentResult> {
    let bytes = state
        .service
        .download_attachment(&mail_id, &attachment_id)
        .await?;
    let target = app
        .dialog()
        .file()
        .set_file_name(&name)
        .blocking_save_file();
    let Some(target) = target else {
        return Ok(SaveAttachmentResult {
            saved: false,
            path: None,
        });
    };
    let path = target
        .into_path()
        .map_err(|_| ApiError::from("the chosen path is not local".to_string()))?;
    std::fs::write(&path, &bytes).map_err(|error| {
        ApiError::from(format!("could not write the attachment: {}", error.kind()))
    })?;
    Ok(SaveAttachmentResult {
        saved: true,
        path: Some(path.to_string_lossy().into_owned()),
    })
}

#[tauri::command]
pub async fn open_external(app: AppHandle, url: String) -> ApiResult<()> {
    let scheme = url
        .split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(scheme.as_str(), "https" | "http" | "mailto") {
        return Err(ApiError::from("blocked link scheme".to_string()));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| ApiError::from(format!("could not open link: {error}")))
}

#[tauri::command]
pub fn quit_app(app: AppHandle, state: State<'_, AppState>) {
    state.quitting.store(true, Ordering::SeqCst);
    app.exit(0);
}

pub fn focus_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn emit_mailto(app: &AppHandle, args: &[String]) {
    if let Some(href) = args.iter().find(|arg| arg.starts_with("mailto:")) {
        if let Some(request) = crate::mailto::parse(href) {
            let _ = app.emit("mailto", request);
        }
    }
}

/// Cold-start `mailto:` is kept until the frontend asks for it, so a slow
/// WebView cannot lose the message.
pub fn store_mailto(app: &AppHandle, args: &[String]) {
    if let Some(href) = args.iter().find(|arg| arg.starts_with("mailto:")) {
        if let Some(request) = crate::mailto::parse(href) {
            if let Ok(mut pending) = app.state::<AppState>().pending_mailto.lock() {
                *pending = Some(request);
            }
        }
    }
}

#[tauri::command]
pub fn take_pending_mailto(state: State<'_, AppState>) -> Option<crate::mailto::MailtoRequest> {
    state
        .pending_mailto
        .lock()
        .ok()
        .and_then(|mut pending| pending.take())
}
