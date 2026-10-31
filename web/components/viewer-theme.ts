export const VIEWER_THEME_STORAGE_KEY = "mlg-view-theme";
export const VIEWER_THEME_FAMILY_STORAGE_KEY = "mlg-view-theme-family";
export const VIEWER_THEME_MODE_STORAGE_KEY = "mlg-view-theme-mode";

export const VIEWER_THEMES = [
  { id: "atomic", label: "Atomic", colorScheme: "light" },
  { id: "atomic-dark", label: "Atomic Dark", colorScheme: "dark" },
  { id: "classic", label: "Classic", colorScheme: "light" },
  { id: "classic-dark", label: "Classic Dark", colorScheme: "dark" },
  { id: "field-guide", label: "Field Guide", colorScheme: "light" },
  {
    id: "field-guide-dark",
    label: "Field Guide Dark",
    colorScheme: "dark",
  },
  { id: "flat-gray", label: "Flat Gray", colorScheme: "light" },
  {
    id: "flat-gray-dark",
    label: "Flat Gray Dark",
    colorScheme: "dark",
  },
  { id: "journal", label: "Journal", colorScheme: "light" },
  { id: "journal-dark", label: "Journal Dark", colorScheme: "dark" },
  { id: "lcd", label: "LCD", colorScheme: "light" },
  { id: "lcd-dark", label: "LCD Dark", colorScheme: "dark" },
  { id: "manuscript", label: "Manuscript", colorScheme: "light" },
  {
    id: "manuscript-dark",
    label: "Manuscript Dark",
    colorScheme: "dark",
  },
  { id: "mono", label: "Mono", colorScheme: "light" },
  { id: "dark", label: "Mono Dark", colorScheme: "dark" },
  { id: "retro", label: "Retro", colorScheme: "light" },
  { id: "retro-dark", label: "Retro Dark", colorScheme: "dark" },
  { id: "sepia", label: "Sepia", colorScheme: "light" },
  { id: "sepia-dark", label: "Sepia Dark", colorScheme: "dark" },
] as const;

export type ViewerTheme = (typeof VIEWER_THEMES)[number]["id"];

export type ViewerThemeMode = "light" | "dark" | "auto";

export interface ViewerThemeFamily {
  id: string;
  label: string;
  lightTheme: ViewerTheme;
  darkTheme: ViewerTheme;
}

export const VIEWER_THEME_FAMILIES: readonly ViewerThemeFamily[] = [
  {
    id: "atomic",
    label: "Atomic",
    lightTheme: "atomic",
    darkTheme: "atomic-dark",
  },
  {
    id: "classic",
    label: "Classic",
    lightTheme: "classic",
    darkTheme: "classic-dark",
  },
  {
    id: "field-guide",
    label: "Field Guide",
    lightTheme: "field-guide",
    darkTheme: "field-guide-dark",
  },
  {
    id: "flat-gray",
    label: "Flat Gray",
    lightTheme: "flat-gray",
    darkTheme: "flat-gray-dark",
  },
  {
    id: "journal",
    label: "Journal",
    lightTheme: "journal",
    darkTheme: "journal-dark",
  },
  {
    id: "lcd",
    label: "LCD",
    lightTheme: "lcd",
    darkTheme: "lcd-dark",
  },
  {
    id: "manuscript",
    label: "Manuscript",
    lightTheme: "manuscript",
    darkTheme: "manuscript-dark",
  },
  {
    id: "mono",
    label: "Mono",
    lightTheme: "mono",
    darkTheme: "dark",
  },
  {
    id: "retro",
    label: "Retro",
    lightTheme: "retro",
    darkTheme: "retro-dark",
  },
  {
    id: "sepia",
    label: "Sepia",
    lightTheme: "sepia",
    darkTheme: "sepia-dark",
  },
] as const;

export const DEFAULT_VIEWER_THEME_FAMILY = "classic";
export const DEFAULT_VIEWER_THEME_MODE: ViewerThemeMode = "light";
export const DEFAULT_VIEWER_THEME: ViewerTheme = "classic";

export function isViewerTheme(
  value: string | null | undefined,
): value is ViewerTheme {
  return VIEWER_THEMES.some((theme) => theme.id === value);
}

export function isViewerThemeFamily(
  value: string | null | undefined,
): boolean {
  return VIEWER_THEME_FAMILIES.some((family) => family.id === value);
}

export function isViewerThemeMode(
  value: string | null | undefined,
): value is ViewerThemeMode {
  return value === "light" || value === "dark" || value === "auto";
}

export function resolveViewerTheme(
  familyId: string,
  mode: ViewerThemeMode,
): ViewerTheme {
  const family =
    VIEWER_THEME_FAMILIES.find((item) => item.id === familyId) ??
    VIEWER_THEME_FAMILIES[1]; // default Classic
  const prefersDark =
    typeof window !== "undefined" &&
    window.matchMedia?.("(prefers-color-scheme: dark)")?.matches;
  const isDark = mode === "dark" || (mode === "auto" && Boolean(prefersDark));
  return isDark ? family.darkTheme : family.lightTheme;
}

export function getFamilyForTheme(theme: ViewerTheme): {
  family: ViewerThemeFamily;
  mode: ViewerThemeMode;
} {
  for (const family of VIEWER_THEME_FAMILIES) {
    if (family.lightTheme === theme) {
      return { family, mode: "light" };
    }
    if (family.darkTheme === theme) {
      return { family, mode: "dark" };
    }
  }
  return { family: VIEWER_THEME_FAMILIES[1], mode: "light" };
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
