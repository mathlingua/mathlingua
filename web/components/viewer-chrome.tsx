"use client";

import { useEffect, useRef, useState } from "react";
import type { OutlineState } from "./outline-state";
import { VIEWER_FONTS, type ViewerFont } from "./viewer-font";
import { VIEWER_THEMES, type ViewerTheme } from "./viewer-theme";
import styles from "./viewer-chrome.module.css";

/** Props for the sticky viewer toolbar. */
interface ViewerChromeProps {
  /** Toggles the outline panel visibility. */
  onToggleOutline: () => void;
  /** Changes the active viewer theme. */
  onThemeChange: (theme: ViewerTheme) => void;
  /** Changes the active typography/font mode. */
  onFontChange: (font: ViewerFont) => void;
  /** Toggles inline expression-type annotations. */
  onToggleTypes: () => void;
  /** Current outline visibility mode. */
  outlineState: OutlineState;
  /** Current visual theme. */
  theme: ViewerTheme;
  /** Current typography/font mode. */
  font: ViewerFont;
  /** Whether inline expression-type annotations are visible. */
  showTypes: boolean;
}

/** Renders the top toolbar for the collection viewer. */
export function ViewerChrome({
  onToggleOutline,
  onThemeChange,
  onFontChange,
  onToggleTypes,
  outlineState,
  theme,
  font,
  showTypes,
}: ViewerChromeProps) {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const settingsMenuRef = useRef<HTMLDivElement>(null);

  const expanded =
    outlineState === "auto" ? undefined : outlineState === "open";

  useEffect(() => {
    if (!isSettingsOpen) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      if (
        event.target instanceof Node &&
        settingsMenuRef.current?.contains(event.target)
      ) {
        return;
      }

      setIsSettingsOpen(false);
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setIsSettingsOpen(false);
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isSettingsOpen]);

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
      <div className={styles.tools}>
        <div className={styles.settingsMenu} ref={settingsMenuRef}>
          <button
            aria-expanded={isSettingsOpen}
            aria-haspopup="dialog"
            aria-label="Settings"
            className={styles.settingsIconButton}
            onClick={() => setIsSettingsOpen((value) => !value)}
            title="Settings"
            type="button"
          >
            <SettingsIcon />
          </button>
          {isSettingsOpen ? (
            <div
              aria-label="Viewer settings"
              className={styles.settingsPopover}
              role="dialog"
            >
              <div className={styles.settingsSection}>
                <div className={styles.settingsSectionTitle}>Font</div>
                <div
                  aria-label="Font"
                  className={styles.segmentedControl}
                  role="radiogroup"
                >
                  {VIEWER_FONTS.map((item) => (
                    <button
                      aria-checked={font === item.id}
                      className={
                        font === item.id
                          ? `${styles.segmentedButton} ${styles.segmentedButtonActive}`
                          : styles.segmentedButton
                      }
                      key={item.id}
                      onClick={() => onFontChange(item.id)}
                      role="radio"
                      type="button"
                    >
                      {item.label}
                    </button>
                  ))}
                </div>
              </div>

              <div className={styles.settingsDivider} />

              <div className={styles.settingsSection}>
                <div className={styles.settingsSectionTitle}>Annotations</div>
                <button
                  aria-checked={showTypes}
                  className={styles.toggleRow}
                  onClick={onToggleTypes}
                  role="switch"
                  type="button"
                >
                  <span className={styles.toggleLabel}>Expression types</span>
                  <span
                    className={
                      showTypes
                        ? `${styles.switch} ${styles.switchActive}`
                        : styles.switch
                    }
                  >
                    <span className={styles.switchThumb} />
                  </span>
                </button>
              </div>

              <div className={styles.settingsDivider} />

              <div className={styles.settingsSection}>
                <div className={styles.settingsSectionTitle}>Theme</div>
                <div className={styles.selectWrapper}>
                  <select
                    aria-label="Select theme"
                    className={styles.select}
                    onChange={(event) =>
                      onThemeChange(event.target.value as ViewerTheme)
                    }
                    value={theme}
                  >
                    {VIEWER_THEMES.map((item) => (
                      <option key={item.id} value={item.id}>
                        {item.label}
                      </option>
                    ))}
                  </select>
                  <span aria-hidden="true" className={styles.selectChevron}>
                    <ChevronDownIcon />
                  </span>
                </div>
              </div>
            </div>
          ) : null}
        </div>
      </div>
    </header>
  );
}

function SettingsIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.settingsIcon}
      fill="none"
      viewBox="0 0 24 24"
    >
      <path
        d="M3 6h3m4 0h11M3 12h11m4 0h3M3 18h5m4 0h9"
        stroke="currentColor"
        strokeLinecap="round"
        strokeWidth="1.7"
      />
      <circle cx="8" cy="6" r="2" stroke="currentColor" strokeWidth="1.7" />
      <circle cx="16" cy="12" r="2" stroke="currentColor" strokeWidth="1.7" />
      <circle cx="10" cy="18" r="2" stroke="currentColor" strokeWidth="1.7" />
    </svg>
  );
}

function ChevronDownIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.selectChevronIcon}
      fill="none"
      viewBox="0 0 16 16"
    >
      <path
        d="m4 6 4 4 4-4"
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
