import { Info, AlertTriangle, XCircle, X } from "lucide-solid"
import { Show } from "solid-js"

export interface AlertProps {
  type: "info" | "warning" | "error"
  message: string
  onDismiss?: () => void
}

export function AlertCallout(props: AlertProps) {
  const icons = {
    info: () => <Info class="w-5 h-5 flex-shrink-0 mt-0.5 text-blue-500" />,
    warning: () => (
      <AlertTriangle class="w-5 h-5 flex-shrink-0 mt-0.5 text-amber-500" />
    ),
    error: () => <XCircle class="w-5 h-5 flex-shrink-0 mt-0.5 text-red-500" />,
  }

  const styles = {
    info: "bg-blue-50 border-blue-100",
    warning: "bg-amber-50 border-amber-100",
    error: "bg-red-50 border-red-100",
  }

  const Icon = icons[props.type]

  return (
    <div
      data-testid={`alert-${props.type}`}
      class={`rounded-lg border p-4 flex gap-3 animate-slide-in ${styles[props.type]}`}
    >
      <Icon />
      <p class="text-sm flex-1 break-words whitespace-pre-wrap min-w-0 max-h-32 overflow-y-auto">
        {props.message}
      </p>
      <Show when={props.onDismiss}>
        <button
          onClick={props.onDismiss}
          class="text-gray-400 hover:text-gray-600 transition-colors"
        >
          <X class="w-4 h-4" />
        </button>
      </Show>
    </div>
  )
}
