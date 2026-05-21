import styles from "./viewer-chrome.module.css";

/** Props for the sticky viewer toolbar. */
interface ViewerChromeProps {
  /** Whether the outline panel is currently open. */
  isOutlineOpen: boolean;
  /** Toggles the outline panel visibility. */
  onToggleOutline: () => void;
}

/** Renders the top toolbar for the collection viewer. */
export function ViewerChrome({
  isOutlineOpen,
  onToggleOutline,
}: ViewerChromeProps) {
  return (
    <header className={styles.chrome}>
      <button
        aria-expanded={isOutlineOpen}
        aria-label={isOutlineOpen ? "Close outline" : "Open outline"}
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
