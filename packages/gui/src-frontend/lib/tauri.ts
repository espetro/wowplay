import { invoke } from '@tauri-apps/api/core';
import { ResultAsync } from 'neverthrow';

export interface AppConfig {
  runner: string | null;
  wow_dir: string | null;
  show_alerts: boolean;
  bottle: string | null;
}

export interface RunnerStatus {
  name: string;
  display_name: string;
  available: boolean;
  path: string | null;
}

export interface ValidationResult {
  valid: boolean;
  wow_exe_found: boolean;
  divxdecoder_patched: boolean;
  message: string;
  severity: string;
}

export interface SetupResult {
  success: boolean;
  messages: string[];
}

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
    (e) => ipcError(String(e))
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
