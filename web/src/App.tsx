import React from 'react';

import styles from './App.module.css';
import { MainPage } from './pages/MainPage';

export const App = () => {
  return (
    <div className={styles.App}>
      <MainPage />
    </div>
  );
}
