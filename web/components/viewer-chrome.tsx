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
import {
  VIEWER_THEME_FAMILIES,
  VIEWER_THEMES,
  type ViewerTheme,
  type ViewerThemeMode,
} from "./viewer-theme";
import { VIEWER_SPACINGS, type ViewerSpacing } from "./viewer-spacing";
import styles from "./viewer-chrome.module.css";

/** Props for the sticky viewer toolbar. */
interface ViewerChromeProps {
  /** Toggles the outline panel visibility. */
  onToggleOutline: () => void;
  /** Changes the active viewer theme family (e.g. "journal", "classic", etc.). */
  onThemeFamilyChange: (familyId: string) => void;
  /** Changes the active viewer theme mode ("light" | "dark" | "auto"). */
  onThemeModeChange: (mode: ViewerThemeMode) => void;
  /** Changes the active typography/font mode. */
  onFontChange: (font: ViewerFont) => void;
  /** Changes the active reader spacing density. */
  onSpacingChange: (spacing: ViewerSpacing) => void;
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
  /** Current theme family id. */
  themeFamily: string;
  /** Current theme mode ("light" | "dark" | "auto"). */
  themeMode: ViewerThemeMode;
  /** Current visual theme. */
  theme: ViewerTheme;
  /** Current typography/font mode. */
  font: ViewerFont;
  /** Current reader spacing density. */
  spacing: ViewerSpacing;
  /** Current reader font scale multiplier. */
  fontScale: ViewerFontScale;
  /** Whether inline expression-type annotations are visible. */
  showTypes: boolean;
}

/** Renders the top toolbar and secondary slide-down settings toolbar. */
export function ViewerChrome({
  onToggleOutline,
  onThemeFamilyChange,
  onThemeModeChange,
  onFontChange,
  onSpacingChange,
  onDecreaseFontScale,
  onIncreaseFontScale,
  onResetFontScale,
  onToggleTypes,
  outlineState,
  themeFamily,
  themeMode,
  theme,
  font,
  spacing,
  fontScale,
  showTypes,
}: ViewerChromeProps) {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isFontDropdownOpen, setIsFontDropdownOpen] = useState(false);
  const [isSpacingDropdownOpen, setIsSpacingDropdownOpen] = useState(false);
  const [isThemeDropdownOpen, setIsThemeDropdownOpen] = useState(false);
  const fontDropdownRef = useRef<HTMLDivElement>(null);
  const spacingDropdownRef = useRef<HTMLDivElement>(null);
  const themeDropdownRef = useRef<HTMLDivElement>(null);
  const activeFontOptionRef = useRef<HTMLButtonElement>(null);
  const activeSpacingOptionRef = useRef<HTMLButtonElement>(null);
  const activeThemeOptionRef = useRef<HTMLButtonElement>(null);
  const subbarRef = useRef<HTMLDivElement>(null);

  const expanded =
    outlineState === "auto" ? undefined : outlineState === "open";
  const activeFont =
    VIEWER_FONTS.find((item) => item.id === font) ?? VIEWER_FONTS[0];
  const activeSpacing =
    VIEWER_SPACINGS.find((item) => item.id === spacing) ?? VIEWER_SPACINGS[0];
  const activeFamily =
    VIEWER_THEME_FAMILIES.find((item) => item.id === themeFamily) ??
    VIEWER_THEME_FAMILIES[1];
  const canDecreaseFontScale = fontScale > MIN_VIEWER_FONT_SCALE;
  const canIncreaseFontScale = fontScale < MAX_VIEWER_FONT_SCALE;
  const isDefaultFontScale = fontScale === DEFAULT_VIEWER_FONT_SCALE;
  const fontScalePercent = Math.round(fontScale * 100);

  useEffect(() => {
    if (!isSettingsOpen) {
      document.documentElement.style.setProperty(
        "--viewer-subbar-height",
        "0px",
      );
      return;
    }

    const updateHeight = () => {
      if (subbarRef.current) {
        const height = subbarRef.current.offsetHeight;
        document.documentElement.style.setProperty(
          "--viewer-subbar-height",
          `${height}px`,
        );
      }
    };

    updateHeight();

    const observer = new ResizeObserver(() => {
      updateHeight();
    });

    if (subbarRef.current) {
      observer.observe(subbarRef.current);
    }

    return () => {
      observer.disconnect();
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
        if (
          isFontDropdownOpen ||
          isSpacingDropdownOpen ||
          isThemeDropdownOpen
        ) {
          setIsFontDropdownOpen(false);
          setIsSpacingDropdownOpen(false);
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
  }, [
    isSettingsOpen,
    isFontDropdownOpen,
    isSpacingDropdownOpen,
    isThemeDropdownOpen,
  ]);

  useEffect(() => {
    if (!isFontDropdownOpen && !isSpacingDropdownOpen && !isThemeDropdownOpen) {
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
        isSpacingDropdownOpen &&
        !spacingDropdownRef.current?.contains(event.target)
      ) {
        setIsSpacingDropdownOpen(false);
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
  }, [isFontDropdownOpen, isSpacingDropdownOpen, isThemeDropdownOpen]);

  useEffect(() => {
    if (isFontDropdownOpen && activeFontOptionRef.current) {
      activeFontOptionRef.current.scrollIntoView({ block: "nearest" });
    }
  }, [isFontDropdownOpen]);

  useEffect(() => {
    if (isSpacingDropdownOpen && activeSpacingOptionRef.current) {
      activeSpacingOptionRef.current.scrollIntoView({ block: "nearest" });
    }
  }, [isSpacingDropdownOpen]);

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
                  setIsSpacingDropdownOpen(false);
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
        ref={subbarRef}
        role="region"
      >
        <div className={styles.subbarContent}>
          {/* Font custom styled dropdown */}
          <div className={styles.subbarGroup}>
            <span className={styles.controlLabel}>Font</span>
            <div
              className={
                isFontDropdownOpen
                  ? `${styles.dropdown} ${styles.dropdownOpen}`
                  : styles.dropdown
              }
              ref={fontDropdownRef}
            >
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
                      setIsSpacingDropdownOpen(false);
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

          {/* Spacing custom styled dropdown */}
          <div className={styles.subbarGroup}>
            <span className={styles.controlLabel}>Spacing</span>
            <div
              className={
                isSpacingDropdownOpen
                  ? `${styles.dropdown} ${styles.dropdownOpen}`
                  : styles.dropdown
              }
              ref={spacingDropdownRef}
            >
              <button
                aria-expanded={isSpacingDropdownOpen}
                aria-haspopup="listbox"
                aria-label={`Spacing: ${activeSpacing.label}`}
                className={
                  isSpacingDropdownOpen
                    ? `${styles.dropdownButton} ${styles.dropdownButtonOpen}`
                    : styles.dropdownButton
                }
                onClick={() => {
                  setIsSpacingDropdownOpen((val) => {
                    const next = !val;
                    if (next) {
                      setIsFontDropdownOpen(false);
                      setIsThemeDropdownOpen(false);
                    }
                    return next;
                  });
                }}
                type="button"
              >
                <span className={styles.dropdownButtonLabel}>
                  {activeSpacing.label}
                </span>
                <span aria-hidden="true" className={styles.dropdownChevron}>
                  <ChevronDownIcon />
                </span>
              </button>
              {isSpacingDropdownOpen ? (
                <div
                  aria-label="Select spacing"
                  className={`${styles.dropdownMenu} ${styles.dropdownMenuLeft}`}
                  role="listbox"
                >
                  {VIEWER_SPACINGS.map((item) => {
                    const isSelected = spacing === item.id;
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
                          onSpacingChange(item.id);
                          setIsSpacingDropdownOpen(false);
                        }}
                        ref={isSelected ? activeSpacingOptionRef : undefined}
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

          {/* Break to second line */}
          <div className={styles.subbarBreak} />

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
            <div
              className={
                isThemeDropdownOpen
                  ? `${styles.dropdown} ${styles.dropdownOpen}`
                  : styles.dropdown
              }
              ref={themeDropdownRef}
            >
              <button
                aria-expanded={isThemeDropdownOpen}
                aria-haspopup="listbox"
                aria-label={`Theme: ${activeFamily.label}`}
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
                      setIsSpacingDropdownOpen(false);
                    }
                    return next;
                  });
                }}
                type="button"
              >
                <span className={styles.dropdownButtonLabel}>
                  {activeFamily.label}
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
                  {VIEWER_THEME_FAMILIES.map((family) => {
                    const isSelected = themeFamily === family.id;
                    return (
                      <button
                        aria-selected={isSelected}
                        className={
                          isSelected
                            ? `${styles.dropdownOption} ${styles.dropdownOptionActive}`
                            : styles.dropdownOption
                        }
                        key={family.id}
                        onClick={() => {
                          onThemeFamilyChange(family.id);
                          setIsThemeDropdownOpen(false);
                        }}
                        ref={isSelected ? activeThemeOptionRef : undefined}
                        role="option"
                        type="button"
                      >
                        <span className={styles.dropdownOptionLabel}>
                          {family.label}
                        </span>
                        {isSelected ? <CheckIcon /> : null}
                      </button>
                    );
                  })}
                </div>
              ) : null}
            </div>

            {/* Light / Dark / Auto switcher */}
            <div
              aria-label="Color mode"
              className={styles.themeModeGroup}
              role="radiogroup"
            >
              <button
                aria-checked={themeMode === "light"}
                className={
                  themeMode === "light"
                    ? `${styles.themeModeButton} ${styles.themeModeButtonActive}`
                    : styles.themeModeButton
                }
                onClick={() => onThemeModeChange("light")}
                role="radio"
                title="Light mode"
                type="button"
              >
                <SunIcon />
                <span className={styles.themeModeLabel}>Light</span>
              </button>
              <button
                aria-checked={themeMode === "dark"}
                className={
                  themeMode === "dark"
                    ? `${styles.themeModeButton} ${styles.themeModeButtonActive}`
                    : styles.themeModeButton
                }
                onClick={() => onThemeModeChange("dark")}
                role="radio"
                title="Dark mode"
                type="button"
              >
                <MoonIcon />
                <span className={styles.themeModeLabel}>Dark</span>
              </button>
              <button
                aria-checked={themeMode === "auto"}
                className={
                  themeMode === "auto"
                    ? `${styles.themeModeButton} ${styles.themeModeButtonActive}`
                    : styles.themeModeButton
                }
                onClick={() => onThemeModeChange("auto")}
                role="radio"
                title="Automatic mode (matches device settings)"
                type="button"
              >
                <AutoModeIcon />
                <span className={styles.themeModeLabel}>Auto</span>
              </button>
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

function SunIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.themeModeIcon}
      fill="none"
      viewBox="0 0 16 16"
    >
      <circle cx="8" cy="8" r="3.2" stroke="currentColor" strokeWidth="1.5" />
      <path
        d="M8 1.5v1.8M8 12.7v1.8M1.5 8h1.8M12.7 8h1.8M3.4 3.4l1.3 1.3M11.3 11.3l1.3 1.3M3.4 12.6l1.3-1.3M11.3 4.7l1.3-1.3"
        stroke="currentColor"
        strokeLinecap="round"
        strokeWidth="1.4"
      />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.themeModeIcon}
      fill="none"
      viewBox="0 0 16 16"
    >
      <path
        d="M13.2 9.5a5.5 5.5 0 1 1-6.7-6.7 4.5 4.5 0 0 0 6.7 6.7z"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.4"
      />
    </svg>
  );
}

function AutoModeIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.themeModeIcon}
      fill="none"
      viewBox="0 0 16 16"
    >
      <circle cx="8" cy="8" r="6" stroke="currentColor" strokeWidth="1.4" />
      <path d="M8 2a6 6 0 0 1 0 12V2z" fill="currentColor" />
    </svg>
  );
}
