import type { OutlineState } from "./outline-state";
import { VIEWER_THEMES, type ViewerTheme } from "./viewer-theme";
import styles from "./viewer-chrome.module.css";

/** Props for the sticky viewer toolbar. */
interface ViewerChromeProps {
  /** Toggles the outline panel visibility. */
  onToggleOutline: () => void;
  /** Changes the active viewer theme. */
  onThemeChange: (theme: ViewerTheme) => void;
  /** Current outline visibility mode. */
  outlineState: OutlineState;
  /** Current visual theme. */
  theme: ViewerTheme;
}

/** Renders the top toolbar for the collection viewer. */
export function ViewerChrome({
  onToggleOutline,
  onThemeChange,
  outlineState,
  theme,
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
      <div aria-label="Theme" className={styles.themeSwitcher} role="group">
        {VIEWER_THEMES.map((item) => (
          <button
            aria-pressed={theme === item.id}
            className={
              theme === item.id
                ? `${styles.themeButton} ${styles.themeButtonActive}`
                : styles.themeButton
            }
            key={item.id}
            onClick={() => onThemeChange(item.id)}
            type="button"
          >
            {item.label}
          </button>
        ))}
      </div>
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
