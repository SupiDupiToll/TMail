import type { Update } from '@tauri-apps/plugin-updater';

export interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

let pending: Update | null = null;

export async function checkForUpdate(): Promise<UpdateInfo | null> {
  const { check } = await import('@tauri-apps/plugin-updater');
  const update = await check({ timeout: 30_000 });
  if (!update) return null;
  if (pending) await pending.close().catch(() => {});
  pending = update;
  return {
    version: update.version,
    date: update.date,
    body: update.body
  };
}

export async function installPendingUpdate(
  onProgress: (percent: number | null) => void
): Promise<void> {
  if (!pending) throw new Error('No update has been checked yet.');
  let total = 0;
  let downloaded = 0;
  await pending.downloadAndInstall((event) => {
    if (event.event === 'Started') {
      total = event.data.contentLength ?? 0;
      onProgress(0);
    } else if (event.event === 'Progress') {
      downloaded += event.data.chunkLength;
      onProgress(total > 0 ? Math.min(99, Math.round((downloaded / total) * 100)) : null);
    } else if (event.event === 'Finished') {
      onProgress(100);
    }
  });
  pending = null;
  const { relaunch } = await import('@tauri-apps/plugin-process');
  await relaunch();
}
