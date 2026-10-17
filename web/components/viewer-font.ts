export const VIEWER_FONT_STORAGE_KEY = "mlg-view-font";

export const VIEWER_FONTS = [
  { id: "default", label: "Default" },
  { id: "euler", label: "Euler" },
] as const;

export type ViewerFont = (typeof VIEWER_FONTS)[number]["id"];

export const DEFAULT_VIEWER_FONT: ViewerFont = "default";

export function isViewerFont(
  value: string | null | undefined,
): value is ViewerFont {
  return VIEWER_FONTS.some((font) => font.id === value);
}

export function applyViewerFont(font: ViewerFont) {
  const root = document.documentElement;
  root.dataset.font = font;
}
