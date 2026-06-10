export function BottleInput(props: {
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <div>
      <label class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1.5 block">
        Bottle
      </label>
      <input
        data-testid="bottle-input"
        type="text"
        value={props.value}
        onInput={(e) => props.onChange(e.currentTarget.value)}
        placeholder="Win10"
        class="w-full border rounded-lg px-3 py-2.5 text-sm bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>
  );
}
