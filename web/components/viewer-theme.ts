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
  { id: "manuscript", label: "Manuscript", colorScheme: "light" },
  {
    id: "manuscript-dark",
    label: "Manuscript Dark",
    colorScheme: "dark",
  },
  { id: "field-guide", label: "Field Guide", colorScheme: "light" },
  {
    id: "field-guide-dark",
    label: "Field Guide Dark",
    colorScheme: "dark",
  },
  { id: "retro", label: "Retro", colorScheme: "light" },
  { id: "retro-dark", label: "Retro Dark", colorScheme: "dark" },
  { id: "lcd", label: "LCD", colorScheme: "light" },
  { id: "lcd-dark", label: "LCD Dark", colorScheme: "dark" },
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
  const root = document.documentElement;
  root.dataset.theme = theme;
  root.style.colorScheme =
    VIEWER_THEMES.find((item) => item.id === theme)?.colorScheme ?? "light";

  // Safari uses the document canvas and `theme-color` while painting around
  // the iPhone notch. Keep both opaque and synchronized with the selected
  // theme, including when a saved theme is restored during initial load.
  const surfaceColor = getComputedStyle(root)
    .getPropertyValue("--surface")
    .trim();
  if (surfaceColor) {
    root.style.backgroundColor = surfaceColor;
    document
      .querySelector<HTMLMetaElement>('meta[name="theme-color"]')
      ?.setAttribute("content", surfaceColor);
  }
}
