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
4. **List**: open a folder with 200+ messages; check newest-first, day marks,
   unread ticks, attachment/confidential flags, long subjects.
5. **Reader**: open a mail with HTML, a table layout, inline styles, inline
   images, a remote pixel, a link and two attachments. Verify: the sender's
   table/CSS layout is preserved, remote pixel blocked, link opens the system
   browser after the preview, attachment downloads and decrypts, body height
   fits. No timestamps or day separators appear in the list.
6. **Read state**: open an unread mail, wait 800 ms, verify unread clears; press
   Undo and verify it returns.
7. **Compose**: choose a sender alias in From, send to the same account with
   subject/body/attachment; verify the message arrives from that alias, the
   attachment decrypts, and the Sent copy appears.
8. **Encryption state**: send to a Tuta address; verify the composer shows
   "Encrypted" and the mail is readable only in Tuta. Send to an external
   address; verify the composer explicitly says "Not encrypted". Password-
   protected external mail is intentionally absent in this build.
9. **Reply/forward**: verify `Re:`/`Fwd:` subjects, recipients, threading in the
   conversation view of the official client.
10. **Moves**: archive, trash, restore via Undo, delete forever from Trash.
11. **Search**: search a subject, a sender and a body word from the reader;
    verify snippets and that opening a hit selects its folder.
12. **Realtime**: from another client, send a mail and move a mail; verify list
    and counters update without a manual refresh.
13. **Offline**: disconnect the network, open a cached mail, restart, open it
    again; verify the cached body and a clear offline state.
14. **Mailto**: `xdg-open 'mailto:a@example.com?subject=Test&body=Hello'` with
    TMail running and stopped; verify the composer opens both times with the
    default sender selected.
15. **Tray/notify**: close the window (hides), restore from tray, quit; verify a
    new-mail notification while backgrounded.
16. **Scale/theme**: 1x, 1.5x, 2x; light/dark/system; 800×600 and 1920×1080.
17. **Version guard**: use an outdated build and confirm the error says to
    update and does not delete the saved session.
