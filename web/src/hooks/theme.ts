
export interface Theme {
  sidebarWidth: number;
  sizeXXSmall: number;
  sizeXSmall: number;
  sizeSmall: number;
  sizeMedium: number;
  sizeLarge: number;
  sizeXLarge: number;
  sizeXXLarge: number;
  border: string;
  background: string;
  mainWidth: number;
}

export function useTheme(): Theme {
  return {
    sidebarWidth: 350,
    sizeXXSmall: 4,
    sizeXSmall: 8,
    sizeSmall: 16,
    sizeMedium: 24,
    sizeLarge: 32,
    sizeXLarge: 48,
    sizeXXLarge: 56,
    border: '#e0e0e0',
    background: '#ffffff',
    mainWidth: 800,
  };
}
