
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

export const LIGHT_THEME: Theme = {
  sidebarWidth: 250,
  sizeXXSmall: 4,
  sizeXSmall: 8,
  sizeSmall: 16,
  sizeMedium: 24,
  sizeLarge: 32,
  sizeXLarge: 48,
  sizeXXLarge: 56,
  border: '#e5e5e5',
  background: '#ffffff',
  mainWidth: 800,
}
