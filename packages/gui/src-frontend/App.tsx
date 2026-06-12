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
  resetConfig,
  setConfig,
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
      async (config) => {
        setStore("config", config)
        if (config.wow_dir) {
          const validResult = await validateWowDir(config.wow_dir)
          validResult.match(
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
    const result = await setConfig(store.config)
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
      setConfig(store.config),
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
          setStore("alerts", [
            { id: "val-err", type: "error", message: result.message },
          ])
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
    const result = await setConfig(store.config)
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
        "Reset all configuration?",
        "This will clear your runner and game folder settings.",
      ),
      (e) => String(e),
    )
    confirmResult.match(
      (confirmed) => {
        if (confirmed) {
          resetConfig().match(
            () => {
              setStore("config", {
                runner: null,
                wow_dir: null,
                show_alerts: true,
                bottle: "Win10",
              })
              setStore("alerts", [])
              setValidation({ valid: false, message: "", severity: "info" })
            },
            (err) =>
              setStore("alerts", [
                { id: "reset-err", type: "error", message: err.message },
              ]),
          )
        }
      },
      (err) =>
        setStore("alerts", [
          { id: "confirm-err", type: "error", message: err },
        ]),
    )
  }

  async function handleToggleAlerts() {
    const newValue = !store.config.show_alerts
    setStore("config", "show_alerts", newValue)
    const result = await setConfig(store.config)
    result.match(
      () => {},
      (err) =>
        setStore("alerts", [
          { id: "save-err", type: "error", message: err.message },
        ]),
    )
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
