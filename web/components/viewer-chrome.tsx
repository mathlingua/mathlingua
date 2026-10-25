"use client";

import { useEffect, useRef, useState } from "react";
import type { OutlineState } from "./outline-state";
import {
  DEFAULT_VIEWER_FONT_SCALE,
  MAX_VIEWER_FONT_SCALE,
  MIN_VIEWER_FONT_SCALE,
  VIEWER_FONTS,
  type ViewerFont,
  type ViewerFontScale,
} from "./viewer-font";
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
  /** Decreases the active reader font scale. */
  onDecreaseFontScale: () => void;
  /** Increases the active reader font scale. */
  onIncreaseFontScale: () => void;
  /** Resets the active reader font scale to default. */
  onResetFontScale: () => void;
  /** Toggles inline expression-type annotations. */
  onToggleTypes: () => void;
  /** Current outline visibility mode. */
  outlineState: OutlineState;
  /** Current visual theme. */
  theme: ViewerTheme;
  /** Current typography/font mode. */
  font: ViewerFont;
  /** Current reader font scale multiplier. */
  fontScale: ViewerFontScale;
  /** Whether inline expression-type annotations are visible. */
  showTypes: boolean;
}

/** Renders the top toolbar and secondary slide-down settings toolbar. */
export function ViewerChrome({
  onToggleOutline,
  onThemeChange,
  onFontChange,
  onDecreaseFontScale,
  onIncreaseFontScale,
  onResetFontScale,
  onToggleTypes,
  outlineState,
  theme,
  font,
  fontScale,
  showTypes,
}: ViewerChromeProps) {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isFontDropdownOpen, setIsFontDropdownOpen] = useState(false);
  const [isThemeDropdownOpen, setIsThemeDropdownOpen] = useState(false);
  const fontDropdownRef = useRef<HTMLDivElement>(null);
  const themeDropdownRef = useRef<HTMLDivElement>(null);
  const activeFontOptionRef = useRef<HTMLButtonElement>(null);
  const activeThemeOptionRef = useRef<HTMLButtonElement>(null);

  const expanded =
    outlineState === "auto" ? undefined : outlineState === "open";
  const activeFont =
    VIEWER_FONTS.find((item) => item.id === font) ?? VIEWER_FONTS[0];
  const activeTheme =
    VIEWER_THEMES.find((item) => item.id === theme) ?? VIEWER_THEMES[0];
  const canDecreaseFontScale = fontScale > MIN_VIEWER_FONT_SCALE;
  const canIncreaseFontScale = fontScale < MAX_VIEWER_FONT_SCALE;
  const isDefaultFontScale = fontScale === DEFAULT_VIEWER_FONT_SCALE;
  const fontScalePercent = Math.round(fontScale * 100);

  useEffect(() => {
    document.documentElement.style.setProperty(
      "--viewer-subbar-height",
      isSettingsOpen ? "42px" : "0px",
    );
    return () => {
      document.documentElement.style.setProperty(
        "--viewer-subbar-height",
        "0px",
      );
    };
  }, [isSettingsOpen]);

  useEffect(() => {
    if (!isSettingsOpen) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        if (isFontDropdownOpen || isThemeDropdownOpen) {
          setIsFontDropdownOpen(false);
          setIsThemeDropdownOpen(false);
        } else {
          setIsSettingsOpen(false);
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isSettingsOpen, isFontDropdownOpen, isThemeDropdownOpen]);

  useEffect(() => {
    if (!isFontDropdownOpen && !isThemeDropdownOpen) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      if (!(event.target instanceof Node)) {
        return;
      }

      if (
        isFontDropdownOpen &&
        !fontDropdownRef.current?.contains(event.target)
      ) {
        setIsFontDropdownOpen(false);
      }

      if (
        isThemeDropdownOpen &&
        !themeDropdownRef.current?.contains(event.target)
      ) {
        setIsThemeDropdownOpen(false);
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
    };
  }, [isFontDropdownOpen, isThemeDropdownOpen]);

  useEffect(() => {
    if (isFontDropdownOpen && activeFontOptionRef.current) {
      activeFontOptionRef.current.scrollIntoView({ block: "nearest" });
    }
  }, [isFontDropdownOpen]);

  useEffect(() => {
    if (isThemeDropdownOpen && activeThemeOptionRef.current) {
      activeThemeOptionRef.current.scrollIntoView({ block: "nearest" });
    }
  }, [isThemeDropdownOpen]);

  return (
    <header className={styles.header}>
      <div className={styles.chrome}>
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
          <button
            aria-controls="viewer-settings-subbar"
            aria-expanded={isSettingsOpen}
            aria-label={
              isSettingsOpen ? "Hide reader settings" : "Show reader settings"
            }
            className={
              isSettingsOpen
                ? `${styles.settingsIconButton} ${styles.settingsIconButtonActive}`
                : styles.settingsIconButton
            }
            onClick={() => {
              setIsSettingsOpen((val) => {
                const next = !val;
                if (!next) {
                  setIsFontDropdownOpen(false);
                  setIsThemeDropdownOpen(false);
                }
                return next;
              });
            }}
            title="Settings"
            type="button"
          >
            <SettingsIcon />
          </button>
        </div>
      </div>

      <div
        aria-hidden={!isSettingsOpen}
        aria-label="Reader settings"
        className={
          isSettingsOpen
            ? `${styles.subbar} ${styles.subbarOpen}`
            : styles.subbar
        }
        id="viewer-settings-subbar"
        role="region"
      >
        <div className={styles.subbarContent}>
          {/* Font custom styled dropdown */}
          <div className={styles.subbarGroup}>
            <span className={styles.controlLabel}>Font</span>
            <div className={styles.dropdown} ref={fontDropdownRef}>
              <button
                aria-expanded={isFontDropdownOpen}
                aria-haspopup="listbox"
                aria-label={`Font: ${activeFont.label}`}
                className={
                  isFontDropdownOpen
                    ? `${styles.dropdownButton} ${styles.dropdownButtonOpen}`
                    : styles.dropdownButton
                }
                onClick={() => {
                  setIsFontDropdownOpen((val) => {
                    const next = !val;
                    if (next) {
                      setIsThemeDropdownOpen(false);
                    }
                    return next;
                  });
                }}
                type="button"
              >
                <span className={styles.dropdownButtonLabel}>
                  {activeFont.label}
                </span>
                <span aria-hidden="true" className={styles.dropdownChevron}>
                  <ChevronDownIcon />
                </span>
              </button>
              {isFontDropdownOpen ? (
                <div
                  aria-label="Select font"
                  className={`${styles.dropdownMenu} ${styles.dropdownMenuLeft}`}
                  role="listbox"
                >
                  {VIEWER_FONTS.map((item) => {
                    const isSelected = font === item.id;
                    return (
                      <button
                        aria-selected={isSelected}
                        className={
                          isSelected
                            ? `${styles.dropdownOption} ${styles.dropdownOptionActive}`
                            : styles.dropdownOption
                        }
                        key={item.id}
                        onClick={() => {
                          onFontChange(item.id);
                          setIsFontDropdownOpen(false);
                        }}
                        ref={isSelected ? activeFontOptionRef : undefined}
                        role="option"
                        type="button"
                      >
                        <span className={styles.dropdownOptionLabel}>
                          {item.label}
                        </span>
                        {isSelected ? <CheckIcon /> : null}
                      </button>
                    );
                  })}
                </div>
              ) : null}
            </div>
            <div
              aria-label="Font size"
              className={styles.fontSizeGroup}
              role="group"
            >
              <button
                aria-disabled={!canDecreaseFontScale}
                aria-label="Decrease font size"
                className={
                  !canDecreaseFontScale
                    ? `${styles.fontSizeButton} ${styles.fontSizeButtonDisabled}`
                    : styles.fontSizeButton
                }
                onClick={() => {
                  if (canDecreaseFontScale) {
                    onDecreaseFontScale();
                  }
                }}
                title="Decrease font size"
                type="button"
              >
                <span aria-hidden="true" className={styles.fontSizeGlyphSmall}>
                  A<span className={styles.fontSizeSign}>−</span>
                </span>
              </button>
              <button
                aria-disabled={isDefaultFontScale}
                aria-label="Reset font size"
                className={
                  isDefaultFontScale
                    ? `${styles.fontSizeButton} ${styles.fontSizeResetButton} ${styles.fontSizeButtonDisabled}`
                    : `${styles.fontSizeButton} ${styles.fontSizeResetButton} ${styles.fontSizeResetButtonActive}`
                }
                onClick={() => {
                  if (!isDefaultFontScale) {
                    onResetFontScale();
                  }
                }}
                title={
                  isDefaultFontScale
                    ? "Default font size (100%)"
                    : `Reset font size (${fontScalePercent}%)`
                }
                type="button"
              >
                <span aria-hidden="true" className={styles.fontSizeGlyphReset}>
                  A
                </span>
              </button>
              <button
                aria-disabled={!canIncreaseFontScale}
                aria-label="Increase font size"
                className={
                  !canIncreaseFontScale
                    ? `${styles.fontSizeButton} ${styles.fontSizeButtonDisabled}`
                    : styles.fontSizeButton
                }
                onClick={() => {
                  if (canIncreaseFontScale) {
                    onIncreaseFontScale();
                  }
                }}
                title="Increase font size"
                type="button"
              >
                <span aria-hidden="true" className={styles.fontSizeGlyphLarge}>
                  A<span className={styles.fontSizeSign}>+</span>
                </span>
              </button>
            </div>
          </div>

          <div className={styles.subbarDivider} />

          {/* Expression types toggle */}
          <div className={styles.subbarGroup}>
            <button
              aria-checked={showTypes}
              className={styles.toggleRow}
              onClick={onToggleTypes}
              role="switch"
              type="button"
            >
              <span className={`${styles.controlLabel} ${styles.labelFull}`}>
                Expression types
              </span>
              <span className={`${styles.controlLabel} ${styles.labelShort}`}>
                Types
              </span>
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

          <div className={styles.subbarDivider} />

          {/* Theme custom styled dropdown */}
          <div className={styles.subbarGroup}>
            <span className={styles.controlLabel}>Theme</span>
            <div className={styles.dropdown} ref={themeDropdownRef}>
              <button
                aria-expanded={isThemeDropdownOpen}
                aria-haspopup="listbox"
                aria-label={`Theme: ${activeTheme.label}`}
                className={
                  isThemeDropdownOpen
                    ? `${styles.dropdownButton} ${styles.dropdownButtonOpen}`
                    : styles.dropdownButton
                }
                onClick={() => {
                  setIsThemeDropdownOpen((val) => {
                    const next = !val;
                    if (next) {
                      setIsFontDropdownOpen(false);
                    }
                    return next;
                  });
                }}
                type="button"
              >
                <span className={styles.dropdownButtonLabel}>
                  {activeTheme.label}
                </span>
                <span aria-hidden="true" className={styles.dropdownChevron}>
                  <ChevronDownIcon />
                </span>
              </button>
              {isThemeDropdownOpen ? (
                <div
                  aria-label="Select theme"
                  className={`${styles.dropdownMenu} ${styles.dropdownMenuRight}`}
                  role="listbox"
                >
                  {VIEWER_THEMES.map((item) => {
                    const isSelected = theme === item.id;
                    return (
                      <button
                        aria-selected={isSelected}
                        className={
                          isSelected
                            ? `${styles.dropdownOption} ${styles.dropdownOptionActive}`
                            : styles.dropdownOption
                        }
                        key={item.id}
                        onClick={() => {
                          onThemeChange(item.id);
                          setIsThemeDropdownOpen(false);
                        }}
                        ref={isSelected ? activeThemeOptionRef : undefined}
                        role="option"
                        type="button"
                      >
                        <span className={styles.dropdownOptionLabel}>
                          {item.label}
                        </span>
                        {isSelected ? <CheckIcon /> : null}
                      </button>
                    );
                  })}
                </div>
              ) : null}
            </div>
          </div>
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
      className={styles.dropdownChevronIcon}
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

function CheckIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.dropdownCheckIcon}
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
