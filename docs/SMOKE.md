# Smoke checklist for the dedicated test account

Run only against a dedicated test account. Never point this at a real mailbox.
Set `TMAIL_TEST_EMAIL` / `TMAIL_TEST_PASSWORD` for the future integration crate;
for now, run the desktop app with `TMAIL_API_URL=https://app.tuta.com`.

1. **Cold sign-in**: password only; verify the keyring entry appears and a
   restart resumes without asking for the password.
2. **TOTP**: enable TOTP on the test account, sign in, verify the code step and
   the 30 s pending poll.
3. **Folders**: system folders plus one nested custom folder; verify order,
   indentation, and unread counts.
4. **List**: open a folder with 200+ messages; check newest-first, avatars,
   bolder unread senders, unread dots, attachment/confidential flags, long
   subjects, and that no timestamp or day label is shown.
5. **Reader/attachment preview**: open a mail with HTML, a table layout, inline styles, inline
   images, a remote pixel, a link and two attachments. Verify: the sender's
   table/CSS layout is preserved, remote pixel blocked, link opens the system
   browser after the preview, attachment downloads and decrypts, body height
   fits. Click a text/CSV attachment to preview it; click an image to preview
   it; PDFs should offer Save. Click both an inline link and a link chip in the
   "Links" strip; both should open the system browser. No timestamps or day
   separators appear in the list, and the mail canvas stays readable in dark
   mode.
6. **Read state**: open an unread mail, wait 800 ms, verify unread clears; press
   Undo and verify it returns.
7. **Compose**: choose a sender alias in From, send to the same account with
   subject/body/attachment; verify the message arrives from that alias, the
   attachment decrypts, and the Sent copy appears. Repeat as a normal external
   send and as a Tuta-recipient E2E send. Schedule one for +5 minutes and verify
   it leaves at the scheduled time.
8. **Layout**: collapse the sidebar to icons and re-expand it with the visible
   chevron, resize both columns by dragging and with arrow keys, expand the
   composer to its large centred window (not fullscreen), then reset the
   layout. Reload and confirm the choices persisted.
9. **Encryption state**: send to a Tuta address; verify the composer shows
   "Encrypted" and the mail is readable only in Tuta. Send to an external
   address; verify the composer explicitly says "Not encrypted". Password-
   protected external mail is intentionally absent in this build.
10. **Reply/forward**: verify `Re:`/`Fwd:` subjects, recipients, threading in the
   conversation view of the official client.
11. **Moves**: archive, trash, restore via Undo, delete forever from Trash.
12. **Search**: search a subject, a sender and a body word from the reader;
    verify snippets and that opening a hit selects its folder.
13. **Realtime**: from another client, send a mail and move a mail; verify list
    and counters update without a manual refresh.
14. **Offline**: disconnect the network, open a cached mail, restart, open it
    again; verify the cached body and a clear offline state.
15. **Mailto**: `xdg-open 'mailto:a@example.com?subject=Test&body=Hello'` with
    TMail running and stopped; verify the composer opens both times with the
    default sender selected.
16. **Tray/notify**: close the window (hides), restore from tray, quit; verify a
    new-mail notification while backgrounded. It must appear once, with the
    TMail icon, sender and subject, and an "Open email" button that opens the
    message in TMail.
17. **Scale/theme**: 1x, 1.5x, 2x; light/dark/system; 800×600 and 1920×1080.
18. **Version guard**: use an outdated build and confirm the error says to
    update and does not delete the saved session.
