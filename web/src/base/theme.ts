
export interface Theme {
  sizes: {
    sidebarWidth: number;
    sizeXXSmall: number;
    sizeXSmall: number;
    sizeSmall: number;
    sizeMedium: number;
    sizeLarge: number;
    sizeXLarge: number;
    sizeXXLarge: number;
    mainWidth: number;
  };
  colors: {
    border: string;
    background: string;
    innerShadow: string;
    outerShadow: string;
    textArgumentColor: string;
    formulationArgumentColor: string;
    idColor: string;
    sectionHeaderColor: string;
    textBlockColor: string;
  };
}

export const LIGHT_THEME: Theme = {
  sizes: {
    sidebarWidth: 250,
    sizeXXSmall: 4,
    sizeXSmall: 8,
    sizeSmall: 16,
    sizeMedium: 24,
    sizeLarge: 32,
    sizeXLarge: 48,
    sizeXXLarge: 56,
    mainWidth: 800,
  },
  colors: {
    border: '#e5e5e5',
    background: '#ffffff',
    innerShadow: 'rgba(50, 50, 105, 0.15)',
    outerShadow: 'rgba(0, 0, 0, 0.05)',
    textArgumentColor: 'black',
    formulationArgumentColor: 'black',
    idColor: 'black',
    sectionHeaderColor: '#05b',
    textBlockColor: 'black',
  },
}
