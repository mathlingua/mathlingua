export const VIEWER_FONT_STORAGE_KEY = "mlg-view-font";
export const VIEWER_FONT_SCALE_STORAGE_KEY = "mlg-view-font-scale";

export const VIEWER_FONTS = [
  { id: "default", label: "Default" },
  { id: "euler", label: "Euler" },
] as const;

export type ViewerFont = (typeof VIEWER_FONTS)[number]["id"];

export const DEFAULT_VIEWER_FONT: ViewerFont = "default";

export const VIEWER_FONT_SCALES = [0.8, 0.9, 1, 1.1, 1.2, 1.3, 1.4] as const;

export type ViewerFontScale = (typeof VIEWER_FONT_SCALES)[number];

export const DEFAULT_VIEWER_FONT_SCALE: ViewerFontScale = 1;
export const MIN_VIEWER_FONT_SCALE: ViewerFontScale = VIEWER_FONT_SCALES[0];
export const MAX_VIEWER_FONT_SCALE: ViewerFontScale =
  VIEWER_FONT_SCALES[VIEWER_FONT_SCALES.length - 1];

export function isViewerFont(
  value: string | null | undefined,
): value is ViewerFont {
  return VIEWER_FONTS.some((font) => font.id === value);
}

export function applyViewerFont(font: ViewerFont) {
  const root = document.documentElement;
  root.dataset.font = font;
}

export function parseViewerFontScale(
  value: string | number | null | undefined,
): ViewerFontScale | null {
  if (value === null || value === undefined || value === "") {
    return null;
  }
  const numeric = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(numeric)) {
    return null;
  }
  const match = VIEWER_FONT_SCALES.find(
    (scale) => Math.abs(scale - numeric) < 0.001,
  );
  return match ?? null;
}

export function stepViewerFontScale(
  current: ViewerFontScale,
  direction: -1 | 1,
): ViewerFontScale {
  const index = VIEWER_FONT_SCALES.indexOf(current);
  const currentIndex =
    index >= 0 ? index : VIEWER_FONT_SCALES.indexOf(DEFAULT_VIEWER_FONT_SCALE);
  const nextIndex = Math.min(
    VIEWER_FONT_SCALES.length - 1,
    Math.max(0, currentIndex + direction),
  );
  return VIEWER_FONT_SCALES[nextIndex];
}

export function applyViewerFontScale(scale: ViewerFontScale) {
  const root = document.documentElement;
  root.style.setProperty("--viewer-font-scale", String(scale));
}
