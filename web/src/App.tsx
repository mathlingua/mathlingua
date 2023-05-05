import React from 'react';

import { MathlinguaContext } from './base/context';
import { MainPage } from './pages/MainPage';
import { LIGHT_THEME } from './base/theme';

export function App() {
  return (
    <MathlinguaContext.Provider value={{
      theme: LIGHT_THEME,
    }}>
      <MainPage />
    </MathlinguaContext.Provider>
  );
}
