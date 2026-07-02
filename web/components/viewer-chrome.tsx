"use client";

import { useEffect, useRef, useState } from "react";
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
  const [isThemeMenuOpen, setIsThemeMenuOpen] = useState(false);
  const themeMenuRef = useRef<HTMLDivElement>(null);
  const expanded =
    outlineState === "auto" ? undefined : outlineState === "open";
  const activeTheme =
    VIEWER_THEMES.find((item) => item.id === theme) ?? VIEWER_THEMES[0];

  useEffect(() => {
    if (!isThemeMenuOpen) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      if (
        event.target instanceof Node &&
        themeMenuRef.current?.contains(event.target)
      ) {
        return;
      }

      setIsThemeMenuOpen(false);
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsThemeMenuOpen(false);
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isThemeMenuOpen]);

  const handleThemeChange = (nextTheme: ViewerTheme) => {
    onThemeChange(nextTheme);
    setIsThemeMenuOpen(false);
  };

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
      <div className={styles.themeMenu} ref={themeMenuRef}>
        <button
          aria-expanded={isThemeMenuOpen}
          aria-haspopup="menu"
          aria-label={`Theme: ${activeTheme.label}`}
          className={styles.themeIconButton}
          onClick={() => setIsThemeMenuOpen((value) => !value)}
          title="Theme"
          type="button"
        >
          <ThemeIcon />
        </button>
        {isThemeMenuOpen ? (
          <div
            aria-label="Select theme"
            className={styles.themePopover}
            role="menu"
          >
            {VIEWER_THEMES.map((item) => (
              <button
                aria-checked={theme === item.id}
                className={
                  theme === item.id
                    ? `${styles.themeOption} ${styles.themeOptionActive}`
                    : styles.themeOption
                }
                key={item.id}
                onClick={() => handleThemeChange(item.id)}
                role="menuitemradio"
                type="button"
              >
                <span className={styles.themeOptionLabel}>{item.label}</span>
                {theme === item.id ? <CheckIcon /> : null}
              </button>
            ))}
          </div>
        ) : null}
      </div>
    </header>
  );
}

function ThemeIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.themeIcon}
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        cx="12"
        cy="12"
        r="8.25"
        stroke="currentColor"
        strokeWidth="1.7"
      />
      <path
        d="M12 3.75a8.25 8.25 0 0 0 0 16.5Z"
        fill="currentColor"
        opacity="0.18"
      />
      <path
        d="M12 3.75a8.25 8.25 0 0 1 0 16.5"
        stroke="currentColor"
        strokeLinecap="round"
        strokeWidth="1.7"
      />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.themeCheckIcon}
      fill="none"
      viewBox="0 0 16 16"
    >
      <path
        d="M3.5 8.2 6.4 11 12.7 4.8"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.7"
      />
    </svg>
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
