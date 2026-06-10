import { createStore } from "solid-js/store"
import type { AppConfig, RunnerStatus } from "../lib/tauri"

interface AlertMessage {
  id: string
  type: "info" | "warning" | "error"
  message: string
}

interface AppStore {
  config: AppConfig
  runners: RunnerStatus[]
  alerts: AlertMessage[]
  isLoading: boolean
}

export const [store, setStore] = createStore<AppStore>({
  config: { runner: null, wow_dir: null, show_alerts: true, bottle: "Win10" },
  runners: [],
  alerts: [],
  isLoading: false,
})

// Derived getters
export const isSetupComplete = () =>
  store.config.runner !== null && store.config.wow_dir !== null
export const visibleAlerts = () =>
  store.alerts.filter((a) => a.type !== "info" || store.config.show_alerts)
export const canLaunch = () => isSetupComplete() && !store.isLoading
