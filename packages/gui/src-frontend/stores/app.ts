import { createStore } from "solid-js/store"
import type { RunnerStatus } from "../lib/tauri"

interface AlertMessage {
  id: string
  type: "info" | "warning" | "error"
  message: string
}

export type StoreConfig = {
  runner: string | null
  wow_dir: string | null
  bottle: string | null
  show_alerts: boolean
}

interface AppStore {
  config: StoreConfig
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
