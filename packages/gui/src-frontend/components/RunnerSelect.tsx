import { Select, createListCollection } from "@ark-ui/solid"
import { For, createMemo } from "solid-js"
import type { RunnerStatus } from "../lib/tauri"

export function RunnerSelect(props: {
  runners: RunnerStatus[]
  value: string | null
  onChange: (value: string) => void
}) {
  const collection = createMemo(() =>
    createListCollection({
      items: props.runners.map((r) => r.name),
      itemToString: (item) =>
        props.runners.find((r) => r.name === item)?.display_name ?? item,
    }),
  )

  return (
    <Select.Root
      collection={collection()}
      value={props.value ? [props.value] : []}
      onValueChange={(e) => props.onChange(e.value[0])}
      positioning={{ placement: "bottom" }}
    >
      <Select.Label class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1.5 block">
        Runner
      </Select.Label>
      <Select.Control>
        <Select.Trigger
          data-testid="runner-select-trigger"
          class="w-full flex items-center justify-between border rounded-lg px-3 py-2.5 text-left hover:border-gray-400 focus:ring-2 focus:ring-primary focus:border-primary transition-all duration-200"
        >
          <Select.ValueText placeholder="Select runner..." />
          <Select.Indicator class="text-gray-400">▼</Select.Indicator>
        </Select.Trigger>
      </Select.Control>
      <Select.Positioner>
        <Select.Content class="bg-white border shadow-lg rounded-lg py-1 min-w-[var(--reference-width)] z-50 animate-slide-in">
          <For each={props.runners}>
            {(runner) => (
              <Select.Item
                item={runner.name}
                data-testid={`runner-option-${runner.name}`}
                class="px-3 py-2 hover:bg-gray-50 cursor-pointer flex items-center justify-between data-[highlighted]:bg-gray-50 transition-colors"
              >
                <Select.ItemText>{runner.display_name}</Select.ItemText>
                <Select.ItemIndicator class="text-primary">
                  ✓
                </Select.ItemIndicator>
              </Select.Item>
            )}
          </For>
        </Select.Content>
      </Select.Positioner>
    </Select.Root>
  )
}
