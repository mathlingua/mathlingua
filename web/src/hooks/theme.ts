
export interface Theme {
  sidebarWidth: number;
  sizeXXSmall: number;
  sizeXSmall: number;
  sizeSmall: number;
  sizeMedium: number;
  sizeLarge: number;
  sizeXLarge: number;
  sizeXXLarge: number;
  gray: string;
}

export function useTheme(): Theme {
  return {
    sidebarWidth: 250,
    sizeXXSmall: 4,
    sizeXSmall: 8,
    sizeSmall: 16,
    sizeMedium: 24,
    sizeLarge: 32,
    sizeXLarge: 48,
    sizeXXLarge: 56,
    gray: '#cccccc',
  };
}