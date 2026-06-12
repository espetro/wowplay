import { invoke } from "@tauri-apps/api/core"
import { ResultAsync } from "neverthrow"

// ── Types (must match wow_silicon_core::config::AppConfig) ──

export interface AppConfig {
  runner: string
  wow_dir: string
  bottle: string
  enable_lib_silicon: boolean
}

export interface RunnerStatus {
  name: string
  display_name: string
  available: boolean
  path: string | null
}

export interface ValidationResult {
  valid: boolean
  wow_exe_found: boolean
  divxdecoder_patched: boolean
  message: string
  severity: string
}

export interface SetupResult {
  success: boolean
  messages: string[]
}

export type TauriError = {
  kind: "ipc"
  message: string
}

function ipcError(message: string): TauriError {
  return { kind: "ipc", message }
}

function invokeResult<T>(
  cmd: string,
  args?: Record<string, unknown>,
): ResultAsync<T, TauriError> {
  return ResultAsync.fromPromise(invoke<T>(cmd, args), (e) =>
    ipcError(
      typeof e === "object" && e !== null && "message" in e
        ? String((e as { message: unknown }).message)
        : String(e),
    ),
  )
}

export const getConfig = () => invokeResult<AppConfig>("get_config")

export const setConfigKey = (key: string, value: string) =>
  invokeResult<string>("set_config", { key, value })

export const listConfig = () => invokeResult<string>("list_config")

export const checkRunners = () => invokeResult<RunnerStatus[]>("check_runners")

export const runSetup = (wowDir: string, runner: string) =>
  invokeResult<SetupResult>("run_setup", { wowDir, runner })

export const launchWow = (wowDir: string, runner: string, bottle: string) =>
  invokeResult<number>("launch_wow", { wowDir, runner, bottle })

export const validateWowDir = (path: string) =>
  invokeResult<ValidationResult>("validate_wow_dir", { path })

export const runReset = (wowDir: string) =>
  invokeResult<{ success: boolean; messages: string[] }>("run_reset", { wowDir })
