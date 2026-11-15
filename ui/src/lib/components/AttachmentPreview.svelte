<script lang="ts">
  import { app } from '../state.svelte';
  import Icon from './Icon.svelte';

  let dialog = $state<HTMLDialogElement | null>(null);

  $effect(() => {
    if (app.preview.open && dialog && !dialog.open) dialog.showModal();
    else if (!app.preview.open && dialog?.open) dialog.close();
  });

  function close(): void {
    app.closePreview();
  }

  function onCancel(event: Event): void {
    event.preventDefault();
    close();
  }

  function save(): void {
    if (app.preview.attachment) {
      void app.saveAttachment(app.preview.attachment.id, app.preview.attachment.name);
    }
  }
</script>

<dialog
  bind:this={dialog}
  class="preview-dialog"
  oncancel={onCancel}
  onclick={(event) => {
    if (event.target === dialog) close();
  }}
  aria-labelledby="preview-title"
>
  <section class="preview">
    <header class="preview-head">
      <span class="file-mark"><Icon name="attachment" size={15} /></span>
      <div class="file-title">
        <h2 id="preview-title">{app.preview.name || 'Attachment'}</h2>
        <span class="faint tiny">{app.preview.mimeType}</span>
      </div>
      <button class="btn btn-sm" onclick={save}>
        <Icon name="download" size={14} />
        Save
      </button>
      <button class="icon-btn" aria-label="Close preview" onclick={close}>
        <Icon name="close" size={17} />
      </button>
    </header>

    <div class="preview-body">
      {#if app.preview.loading}
        <div class="preview-state"><span class="spinner"></span> Loading preview…</div>
      {:else if app.preview.error}
        <div class="preview-state error" role="alert">{app.preview.error}</div>
      {:else if app.preview.kind === 'image'}
        <img class="preview-image" src={app.preview.dataUrl} alt={app.preview.name} />
      {:else if app.preview.kind === 'text'}
        <pre class="preview-text">{app.preview.text}</pre>
      {:else}
        <div class="preview-state">
          <Icon name="attachment" size={28} />
          <strong>Preview not available</strong>
          <span>Save the file to open it in its own application.</span>
        </div>
      {/if}
    </div>
  </section>
</dialog>

<style>
  .preview-dialog {
    width: min(900px, calc(100vw - 40px));
    height: min(720px, calc(100vh - 48px));
    max-width: none;
    max-height: none;
    margin: auto;
    padding: 0;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-lg);
    background: var(--panel);
    color: var(--text);
    box-shadow: var(--shadow);
    overflow: hidden;
  }
  .preview-dialog::backdrop {
    background: var(--scrim);
    backdrop-filter: blur(2px);
  }
  .preview {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .preview-head {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: none;
    padding: 12px 12px 12px 16px;
    border-bottom: 1px solid var(--border);
  }
  .file-mark {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    background: var(--panel-2);
    color: var(--muted);
    flex: none;
  }
  .file-title {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
  }
  .file-title h2 {
    font-size: var(--text-sm);
    font-weight: 680;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .preview-body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 16px;
    background: var(--bg);
  }
  .preview-image {
    display: block;
    max-width: 100%;
    max-height: 100%;
    margin: 0 auto;
    border-radius: var(--radius-sm);
    background: var(--panel);
  }
  .preview-text {
    margin: 0;
    padding: 14px 16px;
    border-radius: var(--radius-sm);
    background: var(--panel);
    color: var(--text);
    font-family: var(--font-mono);
    font-size: 12.5px;
    line-height: 1.55;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .preview-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 100%;
    min-height: 220px;
    color: var(--muted);
    font-size: var(--text-sm);
    text-align: center;
  }
  .preview-state strong {
    color: var(--text);
    font-weight: 680;
  }
  .preview-state.error {
    color: var(--danger);
  }
</style>
