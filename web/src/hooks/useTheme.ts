import React from 'react';

export type Theme = 'light' |
                    'dark' |
                    'sunset' |
                    'high-contrast-light' |
                    'high-contrast-dark';

const LIGHT = {
  '--accent-color': '#05b',
  '--background-color': '#ffffff',
  '--etched-background-color': '#fefefe',
  '--text-color': 'black',
  '--border-color': '#e5e5e5',
  '--box-shadow': 'rgba(50, 50, 105, 0.15) 0px 2px 5px 0px, rgba(0, 0, 0, 0.05) 0px 1px 1px 0px',
  '--etched-box-shadow': 'inset 0 0 5px rgba(50, 50, 105, 0.15)',
  '--formulation-dropdown-icon-color': '#cccccc',
  '--formulation-dropdown-item-hover-color': '#f3f3f3',
  '--top-level-node-kind-menu-icon-color': '#aaaaaa',
  '--source-code-formulation-color': 'darkcyan',
  '--source-code-id-color': '#6052f2',
  '--source-code-text-color': 'green',
  '--header-font-weight': 'none',
  '--selected-tree-node-text-decoration': 'none',
  '--link-color': 'var(--accent-color)',
};


const HIGH_CONTRAST_LIGHT = {
  '--accent-color': 'black',
  '--background-color': 'white',
  '--etched-background-color': 'white',
  '--text-color': 'black',
  '--border-color': 'black',
  '--box-shadow': 'none',
  '--etched-box-shadow': 'none',
  '--formulation-dropdown-icon-color': 'black',
  '--formulation-dropdown-item-hover-color': '#eeeeee',
  '--top-level-node-kind-menu-icon-color': 'black',
  '--source-code-formulation-color': 'black',
  '--source-code-id-color': 'black',
  '--source-code-text-color': 'black',
  '--header-font-weight': 'bold',
  '--selected-tree-node-text-decoration': 'underline',
  '--link-color': 'var(--accent-color)',
};

const SUNSET = {
  '--accent-color': ' #704214',
  '--background-color': '#f0ece7',
  '--etched-background-color': 'color-mix(in srgb, var(--background-color) 100%, var(--accent-color) 1%)',
  '--text-color': '#4e2e0e',
  '--border-color': '#d4c6b8',
  '--box-shadow': '#d4c6b8 0px 2px 5px 0px, #d4c6b826 0px 1px 1px 0px',
  '--etched-box-shadow': 'inset 0 0 5px #d4c6b8',
  '--formulation-dropdown-icon-color': '#b7a089',
  '--formulation-dropdown-item-hover-color': '#e2d9d0',
  '--top-level-node-kind-menu-icon-color': '#9a7a5a',
  '--source-code-formulation-color': '#197361',
  '--source-code-id-color': '#1c5b80',
  '--source-code-text-color': '#1f6319',
  '--header-font-weight': 'bold',
  '--selected-tree-node-text-decoration': 'underline',
  '--link-color': 'var(--accent-color)',
};

const HIGH_CONTRAST_DARK = {
  '--accent-color': 'white',
  '--background-color': 'black',
  '--etched-background-color': 'black',
  '--text-color': 'white',
  '--border-color': 'white',
  '--box-shadow': 'none',
  '--etched-box-shadow': 'none',
  '--formulation-dropdown-icon-color': 'white',
  '--formulation-dropdown-item-hover-color': '#333333',
  '--top-level-node-kind-menu-icon-color': 'white',
  '--source-code-formulation-color': 'white',
  '--source-code-id-color': 'white',
  '--source-code-text-color': 'white',
  '--header-font-weight': 'bold',
  '--selected-tree-node-text-decoration': 'underline',
  '--link-color': 'var(--accent-color)',
};

const DARK = {
  '--accent-color': '#4c6abe',
  '--background-color': '#222326',
  '--etched-background-color': '#202124',
  '--text-color': '#929293',
  '--border-color': '#3d3d3d',
  '--box-shadow': 'rgba(0, 0, 0, 0.5) 0px 2px 5px 0px, rgba(0, 0, 0, 0.05) 0px 1px 1px 0px',
  '--etched-box-shadow': 'inset 0 0 5px rgba(0, 0, 0, 0.5)',
  '--formulation-dropdown-icon-color': '#555555',
  '--formulation-dropdown-item-hover-color': '#404040',
  '--top-level-node-kind-menu-icon-color': '#929293',
  '--source-code-formulation-color': 'rgb(0, 103, 103)',
  '--source-code-id-color': '#453ab9',
  '--source-code-text-color': 'rgb(0, 111, 0)',
  '--header-font-weight': 'none',
  '--selected-tree-node-text-decoration': 'none',
  '--link-color': 'var(--accent-color)',
};

const THEME_STORAGE_KEY = 'MATHLINGUA_THEME';

function themeToCss(theme: Theme): Record<string, string> {
  switch (theme) {
    case 'light':
      return LIGHT;
    case 'dark':
      return DARK;
    case 'sunset':
      return SUNSET;
    case 'high-contrast-light':
      return HIGH_CONTRAST_LIGHT;
    case 'high-contrast-dark':
      return HIGH_CONTRAST_DARK;
    default:
      return LIGHT;
  }
}

export function getNextTheme(theme: Theme): Theme {
  switch (theme) {
    case 'light':
      return 'sunset';
    case 'sunset':
      return 'dark';
    case 'dark':
      return 'high-contrast-dark';
    case 'high-contrast-dark':
      return 'high-contrast-light';
    case 'high-contrast-light':
      return 'light';
    default:
      return 'light';
  }
}

export const useTheme = () => {
  const initialTheme = (localStorage.getItem(THEME_STORAGE_KEY) ?? 'light') as Theme;
  const [theme, setTheme] = React.useState(initialTheme);

  React.useEffect(() => {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
    const css = themeToCss(theme);
    Object.keys(css).forEach((prop) => {
      document.documentElement.style.setProperty(prop, css[prop]);
    });
  }, [theme]);

  return {
    setTheme,
  };
}
