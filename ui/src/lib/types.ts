export interface Address {
  name: string;
  address: string;
}

export interface FolderNode {
  id: string;
  list_id: string;
  entries_list_id: string;
  parent_id: string | null;
  name: string;
  kind: string;
  level: number;
}

export interface MailSummary {
  id: string;
  element_id: string;
  list_id: string;
  subject: string;
  sender: Address;
  received_ms: number;
  unread: boolean;
  has_attachments: boolean;
  attachment_count: number;
  confidential: boolean;
  is_draft: boolean;
  folder_ids: string[];
  conversation_id: string;
  reply_type: number;
  phishing_status: number;
}

export interface AttachmentMeta {
  id: string;
  name: string;
  mime_type: string;
  size: number;
  is_inline: boolean;
  cid: string | null;
}

export interface Recipients {
  to: Address[];
  cc: Address[];
  bcc: Address[];
}

export interface MailDetail {
  summary: MailSummary;
  recipients: Recipients;
  reply_to: Address[];
  sent_ms: number;
  body_html: string;
  body_text: string;
  attachments: AttachmentMeta[];
}

export interface FolderUnread {
  folder_id: string;
  unread: number;
}

export interface Snapshot {
  email: string;
  api_url: string;
  folders: FolderNode[];
  unread: FolderUnread[];
  session_persisted: boolean;
  connected: boolean;
  cached_mails: number;
  cached_bodies: number;
  sender_addresses: string[];
}

export interface SearchHit {
  mail: MailSummary;
  snippet: string;
}

export type LoginResult =
  | { status: 'logged_in'; snapshot: Snapshot }
  | { status: 'totp_required' };

export interface DraftInput {
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  body_html: string;
  plaintext: boolean;
  confidential: boolean;
  confidential_password: string | null;
  sender_address: string | null;
  previous_message_id: string | null;
  conversation_type: number;
}

export interface OutgoingAttachment {
  filename: string;
  mime_type: string;
  /** base64, standard alphabet */
  data: string;
  size: number;
}

export interface SendReceipt {
  message_id: string;
  draft_id: string;
}

export type SyncEvent =
  | { type: 'connected' }
  | { type: 'disconnected' }
  | { type: 'mails_changed'; folder_id: string }
  | { type: 'session_expired' }
  | { type: 'cache_warning'; message: string };

export interface MailtoRequest {
  to: string[];
  cc: string[];
  bcc: string[];
  subject: string;
  body: string;
}

export const CONVERSATION_NEW = 0;
export const CONVERSATION_REPLY = 1;
export const CONVERSATION_FORWARD = 2;
