/**
 * Synthetic backend for browser previews and Playwright tests.
 *
 * All content here is invented and marked as such in the UI. It exists only
 * when the app runs outside Tauri, so a released desktop build never shows it.
 */
import type {
  AttachmentMeta,
  DraftInput,
  FolderNode,
  MailDetail,
  MailSummary,
  SearchHit,
  Snapshot,
  OutgoingAttachment
} from './types';

const now = Date.now();
const day = 86_400_000;

const folders: FolderNode[] = [
  folder('inbox', 'Inbox', 'inbox', 0, 0),
  folder('drafts', 'Drafts', 'draft', 0, 1),
  folder('sent', 'Sent', 'sent', 0, 2),
  folder('trash', 'Trash', 'trash', 0, 3),
  folder('archive', 'Archive', 'archive', 0, 4),
  folder('spam', 'Spam', 'spam', 0, 5),
  folder('work', 'Work', 'custom', 0, 6),
  folder('projects', 'Projects', 'custom', 1, 7, 'work'),
  folder('receipts', 'Receipts', 'custom', 0, 8)
];

function folder(
  id: string,
  name: string,
  kind: string,
  level: number,
  order: number,
  parent: string | null = null
): FolderNode {
  return {
    id,
    list_id: 'folderlist',
    entries_list_id: `entries-${id}`,
    parent_id: parent,
    name,
    kind,
    level
  };
}

const senders = [
  ['Ada Lovelace', 'ada@analytical.example'],
  ['Grace Hopper', 'grace@navy.example'],
  ['Linus Torvalds', 'linus@kernel.example'],
  ['Margaret Hamilton', 'margaret@apollo.example'],
  ['Katherine Johnson', 'katherine@nasa.example'],
  ['Barbara Liskov', 'barbara@mit.example'],
  ['Alan Kay', 'alan@xerox.example'],
  ['Radia Perlman', 'radia@network.example'],
  ['Frances Allen', 'frances@ibm.example'],
  ['Shafi Goldwasser', 'shafi@crypto.example'],
  ['Tuta Support', 'support@tuta.com'],
  ['Deploy Bot', 'deploy@infra.example']
] as const;

const subjects = [
  'Re: Analytical engines and the next release',
  'Quarterly report is ready for review',
  'Lunch on Thursday?',
  'Deploy finished: staging is green',
  'Notes from the architecture review',
  'Invoice 2026-114 attached',
  'Welcome to TMail',
  'Your flight itinerary',
  'Security advisory: rotate your keys',
  'Design sync moved to 14:30',
  'The compiler bug is fixed',
  'Re: keyboard shortcuts proposal',
  'Backup completed',
  'Can you review this patch?',
  'Your subscription renews soon',
  'Weekend reading',
  'Incident postmortem draft',
  'Re: [PATCH v3] faster list sync',
  'New device signed in',
  'Travel receipts for November'
];

const paragraphs = [
  'I read through the proposal twice. The shape is right, but the second phase assumes we can replay events without a cursor — we cannot. Let us keep the cursor store and make the replay explicit.',
  'The numbers for the quarter are attached. Revenue is up, but the support load grew faster than users. I would like to talk about the onboarding flow before we commit to the next hiring plan.',
  'Thursday works. There is a small place near the office that does a decent lunch and is quiet enough to talk. Say 12:30?',
  'Staging is green across all three regions. The migration took 42 seconds and the rollback path has been exercised twice. Production is scheduled for Tuesday 09:00 UTC.',
  'Three things stood out: the cache key derivations are duplicated, the event loop can starve under a large batch, and the HTML sanitiser needs a stricter test matrix. None are blockers for the prototype.',
  'Please find the invoice attached. Payment is due within 14 days. Let me know if you need a different billing address.',
  'This is a synthetic message used by the TMail preview. It exists so the interface can be reviewed without connecting a real mailbox. Nothing here is real.',
  'Your itinerary is confirmed. Check-in opens 24 hours before departure. The booking reference is in the attachment.',
  'We rotated the group keys this morning. If you still see decryption errors after restarting, reply to this message and we will look at the session logs.',
  'The design sync moves to 14:30 to make room for the incident review. Same room, same link.',
  'The bug was a missing bounds check in the cursor decoder. It only appeared with malformed server data, which is exactly the case the fuzz test was meant to catch. Patch is on the branch.',
  'Shortcuts: j/k to move, Enter to open, u for unread, e to archive, # to trash, r/a/f to reply/reply-all/forward, c to compose, Ctrl+K for everything else.',
  'The backup finished at 03:12 and verified clean. Retention is unchanged: 30 daily, 12 monthly.',
  'The patch is small but touches the decryption path, so I would like a second pair of eyes before it lands. Tests cover the AEAD vectors and the legacy fallback.',
  'Your subscription renews on 1 December. You can change the plan or cancel at any time from the account page.',
  'Two essays worth your time this weekend: one on the economics of maintenance, one on why protocols outlive products.',
  'Draft postmortem attached. The summary is deliberately factual; we can add the timeline once the logs are reconciled.',
  'I rebased onto main and dropped the compatibility shim. The list load is now a single batched request instead of one per message. Please re-run the benchmark.',
  'A new device signed in to your account from Arch Linux. If this was you, no action is needed.'
];

const richBody = `<div style="font-family:'Atkinson Hyperlegible Next',sans-serif;max-width:100%;color:#1a211d;background:#ffffff;padding:18px 20px;border-radius:10px">
<h2 style="margin:0 0 10px;font-size:20px;color:#1d6b5f">Your flight itinerary</h2>
<p style="margin:0 0 14px;color:#56625b;font-size:13px">Booking reference <strong>TM-7QK2</strong> · Gate closes 25 minutes before departure.</p>
<table style="width:100%;border-collapse:collapse;font-size:13px">
<tr><th align="left" style="padding:8px 0;border-bottom:1px solid #d5dbd4;color:#56625b">Leg</th><th align="left" style="padding:8px 0;border-bottom:1px solid #d5dbd4;color:#56625b">Departure</th><th align="left" style="padding:8px 0;border-bottom:1px solid #d5dbd4;color:#56625b">Arrival</th></tr>
<tr><td style="padding:8px 0;border-bottom:1px solid #eef1ec">TM 204</td><td style="padding:8px 0;border-bottom:1px solid #eef1ec">Berlin 09:15</td><td style="padding:8px 0;border-bottom:1px solid #eef1ec">Lisbon 11:40</td></tr>
<tr><td style="padding:8px 0">TM 205</td><td style="padding:8px 0">Lisbon 16:05</td><td style="padding:8px 0">Berlin 20:30</td></tr>
</table>
<p style="margin:16px 0 0"><a href="https://example.com/verify" style="color:#1d6b5f;font-weight:700">Manage this booking</a></p>
<p><img src="https://tracker.example/pixel.gif" alt="tracker"><script>window.__pwned = true</script></p>
<p style="margin:14px 0 0;font-size:12px;color:#56625b">Synthetic message for the TMail preview. Not a real booking.</p>
</div>`;

const bodies = paragraphs.map(
  (p, i) => `<p>${p}</p>
<p>${paragraphs[(i + 3) % paragraphs.length]}</p>
<p>— ${senders[i % senders.length][0]}</p>`
);

const mails: MailSummary[] = [];
const details = new Map<string, MailDetail>();

for (let i = 0; i < 240; i++) {
  const sender = senders[i % senders.length];
  const subject = subjects[i % subjects.length];
  const received = now - i * 37 * 60_000 - (i % 7) * day;
  const id = `list-inbox/${String(i).padStart(12, '0')}`;
  const attachmentCount = i === 7 ? 2 : i % 6 === 0 ? (i % 12 === 0 ? 2 : 1) : 0;
  const summary: MailSummary = {
    id,
    element_id: String(i).padStart(12, '0'),
    list_id: 'list-inbox',
    subject: i === 3 ? 'A very long subject line that keeps going and going to test how the row truncates before it collides with the date column' : subject,
    sender: { name: sender[0], address: sender[1] },
    received_ms: received,
    unread: i < 9 || i % 17 === 0,
    has_attachments: attachmentCount > 0,
    attachment_count: attachmentCount,
    confidential: i % 31 === 0,
    is_draft: false,
    folder_ids: [i % 11 === 0 ? 'archive' : i % 13 === 0 ? 'trash' : i % 5 === 0 ? 'work' : 'inbox'],
    conversation_id: `conv-${i}`,
    reply_type: i % 3,
    phishing_status: i === 1 ? 3 : 0
  };
  mails.push(summary);

  const attachments: AttachmentMeta[] = Array.from({ length: attachmentCount }, (_, a) => ({
    id: `${summary.id}/att-${a}`,
    name: a === 0 ? 'quarterly-report.pdf' : 'revenue-by-region.csv',
    mime_type: a === 0 ? 'application/pdf' : 'text/csv',
    size: a === 0 ? 482_133 : 12_004,
    is_inline: false,
    cid: null
  }));

  const detail: MailDetail = {
    summary,
    recipients: {
      to: [{ name: 'You', address: 'you@tuta.com' }],
      cc: i % 4 === 0 ? [{ name: 'Team', address: 'team@example.org' }] : [],
      bcc: []
    },
    reply_to: [],
    sent_ms: received + 120_000,
    conversation_message_id: `<synthetic-${i}@tmail.invalid>`,
    body_html:
      i === 7
        ? richBody
        : `<p>Hello,</p>${bodies[i % bodies.length]}${
            i === 0 || i === 1 || i === 2
              ? '<p><img src="https://tracker.example/pixel.gif" alt="tracker"><script>window.__pwned = true</script><a href="https://example.com/verify">Verify your account</a></p>'
              : ''
          }<p>Thanks,<br>${sender[0]}</p>`,
    body_text: paragraphs[i % paragraphs.length],
    attachments
  };
  details.set(id, detail);
}

// A couple of draft rows so the folder is not empty.
for (let i = 0; i < 2; i++) {
  const id = `list-drafts/draft-${i}`;
  const summary: MailSummary = {
    id,
    element_id: `draft-${i}`,
    list_id: 'list-drafts',
    subject: i === 0 ? 'Draft: reply to Ada' : 'Draft: invoice question',
    sender: { name: 'You', address: 'you@tuta.com' },
    received_ms: now - i * day,
    unread: false,
    has_attachments: false,
    attachment_count: 0,
    confidential: false,
    is_draft: true,
    folder_ids: ['drafts'],
    conversation_id: `draft-conv-${i}`,
    reply_type: 0,
    phishing_status: i === 1 ? 3 : 0
  };
  mails.push(summary);
  details.set(id, {
    summary,
    recipients: { to: [], cc: [], bcc: [] },
    reply_to: [],
    sent_ms: 0,
    conversation_message_id: '',
    body_html: '<p>Draft body…</p>',
    body_text: 'Draft body',
    attachments: []
  });
}

let savedDrafts: MailSummary[] = [];

function summaryForFolder(folderId: string): MailSummary[] {
  if (folderId === 'drafts') return [...savedDrafts, ...mails.filter((m) => m.is_draft)];
  const custom = folderId === 'projects' ? 'work' : folderId;
  return mails
    .filter((m) => m.folder_ids.includes(custom))
    .sort((a, b) => b.received_ms - a.received_ms);
}

export const mock = {
  signedIn: false,
  email: 'you@tuta.com',

  login(): Snapshot {
    this.signedIn = true;
    return this.snapshot();
  },

  snapshot(): Snapshot {
    const unread = folders
      .map((f) => ({
        folder_id: f.id,
        unread: summaryForFolder(f.id).filter((m) => m.unread).length
      }))
      .filter((u) => u.unread > 0);
    return {
      email: this.email,
      api_url: 'https://app.tuta.com',
      folders,
      unread,
      session_persisted: true,
      connected: true,
      cached_mails: mails.length,
      cached_bodies: 12,
      sender_addresses: ['you@tuta.com', 'hello@aliases.example']
    };
  },

  folder(folderId: string, limit = 200): MailSummary[] {
    return summaryForFolder(folderId).slice(0, limit);
  },

  mail(id: string): MailDetail {
    const detail = details.get(id);
    if (!detail) throw new Error('not found');
    detail.summary.unread = false;
    const original = mails.find((m) => m.id === id);
    if (original) original.unread = false;
    return structuredClone(detail);
  },

  search(query: string, limit = 50): SearchHit[] {
    const q = query.toLowerCase().trim();
    if (!q) return [];
    return mails
      .filter(
        (m) =>
          m.subject.toLowerCase().includes(q) ||
          m.sender.name.toLowerCase().includes(q) ||
          m.sender.address.toLowerCase().includes(q) ||
          (details.get(m.id)?.body_text.toLowerCase().includes(q) ?? false)
      )
      .slice(0, limit)
      .map((mail) => ({
        mail,
        snippet: (details.get(mail.id)?.body_text ?? '').slice(0, 160)
      }));
  },

  setUnread(ids: string[], unread: boolean) {
    for (const id of ids) {
      const mail = mails.find((m) => m.id === id);
      if (mail) mail.unread = unread;
    }
  },

  move(ids: string[], target: string) {
    for (const mail of mails) {
      if (ids.includes(mail.id)) {
        mail.folder_ids = [target];
      }
    }
  },

  remove(ids: string[]) {
    for (const id of ids) {
      const mail = mails.find((m) => m.id === id);
      if (mail) mail.folder_ids = ['trash'];
    }
  },

  send(draft: DraftInput, attachments: OutgoingAttachment[]) {
    const id = `list-sent/${Date.now()}`;
    const summary: MailSummary = {
      id,
      element_id: String(Date.now()),
      list_id: 'list-sent',
      subject: draft.subject || '(no subject)',
      sender: { name: 'You', address: this.email },
      received_ms: Date.now(),
      unread: false,
      has_attachments: attachments.length > 0,
      attachment_count: attachments.length,
      confidential: draft.confidential,
      is_draft: false,
      folder_ids: ['sent'],
      conversation_id: `sent-${Date.now()}`,
      reply_type: 0,
      phishing_status: 0
    };
    mails.unshift(summary);
    details.set(id, {
      summary,
      recipients: {
        to: draft.to.map((address) => ({ name: address, address })),
        cc: [],
        bcc: []
      },
      reply_to: [],
      sent_ms: Date.now(),
      conversation_message_id: `<${Date.now()}@tmail.invalid>`,
      body_html: draft.body_html,
      body_text: draft.body_html.replace(/<[^>]+>/g, ' '),
      attachments: []
    });
    return { message_id: `<${Date.now()}@tmail.invalid>`, draft_id: id };
  },

  saveDraft(draft: DraftInput): string {
    const id = `list-drafts/draft-${Date.now()}`;
    const summary: MailSummary = {
      id,
      element_id: String(Date.now()),
      list_id: 'list-drafts',
      subject: draft.subject || '(no subject)',
      sender: { name: 'You', address: this.email },
      received_ms: Date.now(),
      unread: false,
      has_attachments: false,
      attachment_count: 0,
      confidential: false,
      is_draft: true,
      folder_ids: ['drafts'],
      conversation_id: `draft-${Date.now()}`,
      reply_type: 0,
      phishing_status: 0
    };
    savedDrafts = [summary, ...savedDrafts];
    return id;
  },

  saveAttachment(name: string) {
    const body = `Synthetic attachment from the TMail preview.\nFile: ${name}\nThis is not real data.\n`;
    const blob = new Blob([body], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    a.click();
    URL.revokeObjectURL(url);
  }
};
