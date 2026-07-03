export const VIEWER_THEME_STORAGE_KEY = "mlg-view-theme";

export const VIEWER_THEMES = [
  { id: "classic", label: "Classic" },
  { id: "mono", label: "Mono" },
  { id: "sepia", label: "Sepia" },
  { id: "retro", label: "Retro" },
  { id: "dark", label: "Dark" },
] as const;

export type ViewerTheme = (typeof VIEWER_THEMES)[number]["id"];

export const DEFAULT_VIEWER_THEME: ViewerTheme = "classic";

export function isViewerTheme(
  value: string | null | undefined,
): value is ViewerTheme {
  return VIEWER_THEMES.some((theme) => theme.id === value);
}

export function applyViewerTheme(theme: ViewerTheme) {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme =
    theme === "dark" ? "dark" : "light";
}
