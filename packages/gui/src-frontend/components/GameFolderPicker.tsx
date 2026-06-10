import { open } from '@tauri-apps/plugin-dialog';
import { ResultAsync } from 'neverthrow';
import { Folder } from 'lucide-solid';

export function GameFolderPicker(props: {
  value: string | null;
  onChange: (path: string) => void;
  onError?: (msg: string) => void;
}) {
  async function handleBrowse() {
    const result = await ResultAsync.fromPromise(
      open({ directory: true, multiple: false, title: 'Select WoW 3.3.5a Folder' }),
      (e) => String(e),
    );
    result.match(
      (selected) => { if (typeof selected === 'string') props.onChange(selected); },
      (err) => props.onError?.(err),
    );
  }

  return (
    <div>
      <label class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1.5 block">
        Game Folder
      </label>
      <div class="flex gap-2">
        <div class="flex-1 flex items-center gap-2 border rounded-lg px-3 py-2.5 bg-gray-50 min-w-0">
          <Folder class="w-4 h-4 text-gray-400 flex-shrink-0" />
          <span class="truncate text-sm text-gray-700">
            {props.value || 'Select your WoW 3.3.5a folder...'}
          </span>
        </div>
        <button
          data-testid="folder-browse-btn"
          onClick={handleBrowse}
          class="px-4 py-2.5 bg-gray-100 hover:bg-gray-200 rounded-lg text-sm font-medium transition-colors duration-200 active:scale-[0.98] shrink-0"
        >
          Browse
        </button>
      </div>
    </div>
  );
}
