export function ActionButton(props: {
  variant: 'setup' | 'run';
  disabled: boolean;
  loading: boolean;
  onClick: () => void;
}) {
  const label = () => {
    if (props.loading) return 'Loading...';
    return props.variant === 'setup' ? 'Setup' : 'Run';
  };

  return (
    <button
      data-testid="action-btn"
      disabled={props.disabled || props.loading}
      onClick={props.onClick}
      class={`w-full py-3 px-4 rounded-lg font-medium text-sm transition-all duration-200
        ${props.disabled || props.loading
          ? 'bg-gray-200 text-gray-400 cursor-not-allowed'
          : 'bg-primary text-white hover:bg-primary-hover active:scale-[0.98] shadow-sm hover:shadow-md'
        }`}
    >
      {label()}
    </button>
  );
}
