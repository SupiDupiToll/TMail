/**
 * The only script allowed inside the mail frame.
 *
 * It runs in a sandboxed, opaque-origin document (`allow-scripts`, no
 * `allow-same-origin`) and forwards link clicks, height changes, hover state
 * and the reader's keyboard shortcuts to the trusted parent. It has no access
 * to app storage, the Tauri IPC or the parent DOM.
 *
 * The CSP hash is pinned in `BOOTSTRAP_HASH` and in the Tauri CSP. If this
 * script changes, update both hashes; the sanitiser test verifies they match.
 */
export const BOOTSTRAP_SCRIPT = `
(function () {
  var post = function (type, data) {
    var message = { type: type };
    if (data) {
      for (var key in data) message[key] = data[key];
    }
    parent.postMessage(message, '*');
  };
  var anchor = function (event) {
    var target = event.target;
    return target && target.closest ? target.closest('a[data-href]') : null;
  };
  document.addEventListener('click', function (event) {
    var link = anchor(event);
    if (!link) return;
    event.preventDefault();
    post('tmail-link', { href: link.getAttribute('data-href') || '' });
  });
  document.addEventListener('auxclick', function (event) {
    if (event.button !== 1) return;
    var link = anchor(event);
    if (!link) return;
    event.preventDefault();
    post('tmail-link', { href: link.getAttribute('data-href') || '' });
  });
  document.addEventListener('mouseover', function (event) {
    var link = anchor(event);
    post('tmail-hover', { href: link ? link.getAttribute('data-href') || '' : '' });
  });
  document.addEventListener('mouseleave', function () {
    post('tmail-hover', { href: '' });
  });
  document.addEventListener('keydown', function (event) {
    var key = event.key;
    var action = ['j', 'k', 'u', 'e', '#', 'r', 'a', 'f', 'c', 'x', 'g', 'R', 'Delete', 'Escape', '/', '?'].indexOf(key) !== -1 || (event.ctrlKey && key.toLowerCase() === 'k');
    if (!action) return;
    post('tmail-key', { key: key, ctrl: event.ctrlKey, shift: event.shiftKey });
    if (key === 'Escape') event.preventDefault();
  });
  var measure = function () {
    post('tmail-height', {
      height: Math.max(document.body.scrollHeight, document.documentElement.scrollHeight)
    });
  };
  addEventListener('load', measure);
  if ('ResizeObserver' in window) new ResizeObserver(measure).observe(document.body);
})();
`;

/** Filled by the hash check; see the comment above. */
export const BOOTSTRAP_HASH = 'sha256-8gwRP+guhJ0UPsPbea5fO151gN5wxuJoTFhkNwsnomE=';
