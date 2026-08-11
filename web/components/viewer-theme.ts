export const VIEWER_THEME_STORAGE_KEY = "mlg-view-theme";

export const VIEWER_THEMES = [
  { id: "classic", label: "Classic", colorScheme: "light" },
  { id: "classic-dark", label: "Classic Dark", colorScheme: "dark" },
  { id: "mono", label: "Mono", colorScheme: "light" },
  { id: "dark", label: "Mono Dark", colorScheme: "dark" },
  { id: "flat-gray", label: "Flat Gray", colorScheme: "light" },
  {
    id: "flat-gray-dark",
    label: "Flat Gray Dark",
    colorScheme: "dark",
  },
  { id: "sepia", label: "Sepia", colorScheme: "light" },
  { id: "sepia-dark", label: "Sepia Dark", colorScheme: "dark" },
  { id: "retro", label: "Retro", colorScheme: "light" },
  { id: "retro-dark", label: "Retro Dark", colorScheme: "dark" },
  { id: "atomic", label: "Atomic", colorScheme: "light" },
  {
    id: "atomic-dark",
    label: "Atomic Dark",
    colorScheme: "dark",
  },
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
    VIEWER_THEMES.find((item) => item.id === theme)?.colorScheme ?? "light";
}
