import { Menu } from "@ark-ui/solid"
import { MoreHorizontal } from "lucide-solid"

export function OptionsMenu(props: {
  showAlerts: boolean
  onToggleAlerts: () => void
  onFeedback: () => void
  onReset: () => void
}) {
  return (
    <Menu.Root positioning={{ placement: "bottom-end" }}>
      <Menu.Trigger
        data-testid="options-menu-btn"
        class="p-2 hover:bg-gray-100 rounded-lg transition-colors duration-200"
      >
        <MoreHorizontal class="w-5 h-5 text-gray-500" />
      </Menu.Trigger>
      <Menu.Positioner>
        <Menu.Content class="bg-white border shadow-lg rounded-lg py-1 min-w-[160px] animate-slide-in">
          <Menu.Item
            value="toggle-alerts"
            data-testid="menu-item-toggle-alerts"
            onClick={props.onToggleAlerts}
            class="px-4 py-2 hover:bg-gray-50 cursor-pointer text-sm flex justify-between items-center transition-colors"
          >
            <span>Show alerts</span>
            <span>{props.showAlerts ? "✓" : ""}</span>
          </Menu.Item>
          <Menu.Item
            value="feedback"
            data-testid="menu-item-feedback"
            onClick={props.onFeedback}
            class="px-4 py-2 hover:bg-gray-50 cursor-pointer text-sm transition-colors"
          >
            Feedback
          </Menu.Item>
          <Menu.Separator class="border-t border-gray-200 my-1" />
          <Menu.Item
            value="reset"
            data-testid="menu-item-reset"
            onClick={props.onReset}
            class="px-4 py-2 hover:bg-gray-50 cursor-pointer text-sm text-red-600 transition-colors"
          >
            Reset
          </Menu.Item>
        </Menu.Content>
      </Menu.Positioner>
    </Menu.Root>
  )
}
