import { invoke } from '@tauri-apps/api/core';
import { ResultAsync } from 'neverthrow';

import type { AppConfig, RunnerStatus, ValidationResult, SetupResult } from '../gen/bindings';

export type { AppConfig, RunnerStatus, ValidationResult, SetupResult };

export type TauriError = {
  kind: 'ipc';
  message: string;
};

function ipcError(message: string): TauriError {
  return { kind: 'ipc', message };
}

/**
 * Wraps a Tauri IPC invoke in a ResultAsync.
 * Never throws — all errors are captured as `TauriError`.
 */
function invokeResult<T>(cmd: string, args?: Record<string, unknown>): ResultAsync<T, TauriError> {
  return ResultAsync.fromPromise(
    invoke<T>(cmd, args),
    (e) =>
      ipcError(
        typeof e === 'object' && e !== null && 'message' in e
          ? String((e as { message: unknown }).message)
          : String(e),
      ),
  );
}

export const getConfig = () => invokeResult<AppConfig>('get_config');
export const setConfig = (config: AppConfig) => invokeResult<void>('set_config', { config });
export const checkRunners = () => invokeResult<RunnerStatus[]>('check_runners');
export const runSetup = (wowDir: string, runner: string) =>
  invokeResult<SetupResult>('run_setup', { wowDir, runner });
export const launchWow = (wowDir: string, runner: string, bottle: string) =>
  invokeResult<number>('launch_wow', { wowDir, runner, bottle });
export const validateWowDir = (path: string) =>
  invokeResult<ValidationResult>('validate_wow_dir', { path });
export const resetConfig = () => invokeResult<void>('reset_config');
