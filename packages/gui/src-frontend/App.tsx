import { onMount, createSignal, For, Show } from "solid-js"
import { confirm } from "@tauri-apps/plugin-dialog"
import { open } from "@tauri-apps/plugin-shell"
import { ResultAsync } from "neverthrow"
import {
  getConfig,
  checkRunners,
  validateWowDir,
  runSetup,
  launchWow,
  setConfigKey,
  runReset,
} from "./lib/tauri"
import { store, setStore, isSetupComplete, visibleAlerts } from "./stores/app"
import { RunnerSelect } from "./components/RunnerSelect"
import { GameFolderPicker } from "./components/GameFolderPicker"
import { BottleInput } from "./components/BottleInput"
import { AlertCallout } from "./components/AlertCallout"
import { OptionsMenu } from "./components/OptionsMenu"
import { ActionButton } from "./components/ActionButton"

export default function App() {
  const [validation, setValidation] = createSignal({
    valid: false,
    message: "",
    severity: "info",
  })
  const [isReady, setIsReady] = createSignal(false)

  onMount(async () => {
    const configResult = await getConfig()
    configResult.match(
      (config) => {
        setStore("config", {
          runner: config.runner || null,
          wow_dir: config.wow_dir || null,
          bottle: config.bottle || "Win10",
        })
        if (config.wow_dir) {
          validateWowDir(config.wow_dir).match(
            (result) => setValidation(result),
            (err) =>
              setStore("alerts", [
                { id: "val-err", type: "error", message: err.message },
              ]),
          )
        }
      },
      (err) =>
        setStore("alerts", [
          { id: "cfg-err", type: "error", message: err.message },
        ]),
    )

    const runnersResult = await checkRunners()
    runnersResult.match(
      (runners) => setStore("runners", runners),
      (err) =>
        setStore("alerts", [
          { id: "runner-err", type: "error", message: err.message },
        ]),
    )

    setIsReady(true)
  })

  async function handleRunnerChange(runner: string) {
    setStore("config", "runner", runner)
    const result = await setConfigKey("runner", runner)
    result.match(
      () => {},
      (err) =>
        setStore("alerts", [
          { id: "save-err", type: "error", message: err.message },
        ]),
    )
  }

  async function handleFolderChange(path: string) {
    setStore("config", "wow_dir", path)
    const [saveResult, validResult] = await Promise.all([
      setConfigKey("wow_dir", path),
      validateWowDir(path),
    ])
    saveResult.match(
      () => {},
      (err) =>
        setStore("alerts", [
          { id: "save-err", type: "error", message: err.message },
        ]),
    )
    validResult.match(
      (result) => {
        setValidation(result)
        if (!result.valid) {
          setStore("alerts", [{ id: "val-err", type: "error", message: result.message }])
        } else {
          setStore("alerts", [])
        }
      },
      (err) =>
        setStore("alerts", [
          { id: "val-err", type: "error", message: err.message },
        ]),
    )
  }

  function handleFolderError(msg: string) {
    setStore("alerts", [{ id: "browse-err", type: "error", message: msg }])
  }

  async function handleSetup() {
    setStore("isLoading", true)
    try {
      const result = await runSetup(store.config.wow_dir!, store.config.runner!)
      result.match(
        () => {
          setStore("alerts", [
            { id: "setup-ok", type: "info", message: "Setup complete" },
          ])
        },
        (err) => {
          setStore("alerts", [
            { id: "setup-err", type: "error", message: err.message },
          ])
        },
      )
    } finally {
      setStore("isLoading", false)
    }
  }

  async function handleBottleChange(value: string) {
    setStore("config", "bottle", value)
    const result = await setConfigKey("bottle", value)
    result.match(
      () => {},
      (err) =>
        setStore("alerts", [
          { id: "save-err", type: "error", message: err.message },
        ]),
    )
  }

  async function handleRun() {
    setStore("isLoading", true)
    try {
      const result = await launchWow(
        store.config.wow_dir!,
        store.config.runner!,
        store.config.bottle ?? "Win10",
      )
      result.match(
        (pid) => {
          setStore("alerts", [
            {
              id: "run-ok",
              type: "info",
              message: `WoW launched (PID: ${pid})`,
            },
          ])
        },
        (err) => {
          setStore("alerts", [
            { id: "run-err", type: "error", message: err.message },
          ])
        },
      )
    } finally {
      setStore("isLoading", false)
    }
  }

  async function handleReset() {
    const confirmResult = await ResultAsync.fromPromise(
      confirm(
        "Reset all patches and configuration?",
        "This will remove all wowplay patches from your WoW folder and clear configuration.",
      ),
      (e) => String(e),
    )
    confirmResult.match(
      (confirmed) => {
        if (confirmed && store.config.wow_dir) {
          runReset(store.config.wow_dir).match(
            () => {
              setStore("config", {
                runner: null,
                wow_dir: null,
                bottle: "Win10",
              })
              setStore("alerts", [
                { id: "reset-ok", type: "info", message: "Reset complete — patches removed." },
              ])
              setValidation({ valid: false, message: "", severity: "info" })
            },
            (err) =>
              setStore("alerts", [
                { id: "reset-err", type: "error", message: err.message },
              ]),
          )
        } else if (confirmed && !store.config.wow_dir) {
          setStore("alerts", [
            { id: "reset-no-dir", type: "error", message: "No WoW directory configured — cannot reset patches." },
          ])
        }
      },
      (err) =>
        setStore("alerts", [
          { id: "confirm-err", type: "error", message: String(err) },
        ]),
    )
  }

  function handleToggleAlerts() {
    const newValue = !store.config.show_alerts
    setStore("config", "show_alerts", newValue)
  }

  function handleFeedback() {
    open("https://tally.so/r/9q4LJQ")
  }

  return (
    <div class="h-screen flex flex-col p-5 gap-4 select-none overflow-hidden">
      <div class="flex justify-between items-start shrink-0">
        <div>
          <h1 class="text-xl font-semibold text-gray-900">WoW on Silicon</h1>
          <p class="text-gray-500 text-sm mt-0.5">
            Configure your Apple Silicon WoW runner and launch settings.
          </p>
        </div>
        <OptionsMenu
          showAlerts={store.config.show_alerts}
          onToggleAlerts={handleToggleAlerts}
          onFeedback={handleFeedback}
          onReset={handleReset}
        />
      </div>

      <hr class="border-gray-200 shrink-0" />

      <div class="flex flex-col gap-3 shrink-0">
        <RunnerSelect
          runners={store.runners}
          value={store.config.runner}
          onChange={handleRunnerChange}
        />
        <GameFolderPicker
          value={store.config.wow_dir}
          onChange={handleFolderChange}
          onError={handleFolderError}
        />
        <BottleInput
          value={store.config.bottle ?? "Win10"}
          onChange={handleBottleChange}
        />
      </div>

      <Show when={visibleAlerts().length > 0}>
        <div class="flex flex-col gap-2 overflow-y-auto min-h-0 min-w-0 overflow-x-hidden">
          <For each={visibleAlerts()}>
            {(alert) => (
              <AlertCallout
                type={alert.type}
                message={alert.message}
                {...(alert.type === "info"
                  ? { onDismiss: () => setStore("alerts", []) }
                  : {})}
              />
            )}
          </For>
        </div>
      </Show>

      <div class="flex-1 min-h-0" />

      <hr class="border-gray-200 shrink-0" />

      <ActionButton
        variant={isSetupComplete() ? "run" : "setup"}
        disabled={
          !isReady() ||
          !store.config.runner ||
          !store.config.wow_dir ||
          !validation().valid
        }
        loading={store.isLoading}
        onClick={() => (isSetupComplete() ? handleRun() : handleSetup())}
      />
    </div>
  )
}
