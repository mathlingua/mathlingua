export const VIEWER_SPACING_STORAGE_KEY = "mlg-view-spacing";

export const VIEWER_SPACINGS = [
  { id: "compact", label: "Compact" },
  { id: "comfortable", label: "Comfortable" },
  { id: "spacious", label: "Spacious" },
] as const;

export type ViewerSpacing = (typeof VIEWER_SPACINGS)[number]["id"];

export const DEFAULT_VIEWER_SPACING: ViewerSpacing = "comfortable";

export function isViewerSpacing(
  value: string | null | undefined,
): value is ViewerSpacing {
  return VIEWER_SPACINGS.some((spacing) => spacing.id === value);
}

export function applyViewerSpacing(spacing: ViewerSpacing) {
  const root = document.documentElement;
  root.dataset.spacing = spacing;
}
