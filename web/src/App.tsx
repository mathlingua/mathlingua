import React from 'react';

import { Routes, Route, BrowserRouter } from "react-router-dom";

import styles from './App.module.css';
import { MainPage } from './pages/MainPage';

export const App = () => {
  return (
    <BrowserRouter>
      <div className={styles.App}>
        <Routes>
          <Route path="/*" element={<MainPage />} />
        </Routes>
      </div>
    </BrowserRouter>
  );
}
