import type { OutlineState } from "./outline-state";
import styles from "./viewer-chrome.module.css";

/** Props for the sticky viewer toolbar. */
interface ViewerChromeProps {
  /** Toggles the outline panel visibility. */
  onToggleOutline: () => void;
  /** Current outline visibility mode. */
  outlineState: OutlineState;
}

/** Renders the top toolbar for the collection viewer. */
export function ViewerChrome({
  onToggleOutline,
  outlineState,
}: ViewerChromeProps) {
  const expanded =
    outlineState === "auto" ? undefined : outlineState === "open";

  return (
    <header className={styles.chrome}>
      <button
        aria-expanded={expanded}
        aria-label={outlineButtonLabel(outlineState)}
        className={styles.button}
        onClick={onToggleOutline}
        type="button"
      >
        <svg
          aria-hidden="true"
          fill="none"
          height="22"
          viewBox="0 0 24 24"
          width="22"
        >
          <path
            d="M4 6.5H20M4 12H20M4 17.5H20"
            stroke="currentColor"
            strokeLinecap="round"
            strokeWidth="1.9"
          />
        </svg>
      </button>
      <div className={styles.spacer} />
    </header>
  );
}

function outlineButtonLabel(outlineState: OutlineState): string {
  switch (outlineState) {
    case "open":
      return "Close outline";
    case "closed":
      return "Open outline";
    case "auto":
      return "Toggle outline";
  }
}
