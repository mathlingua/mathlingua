import { configureStore, ThunkAction, Action } from '@reduxjs/toolkit';

import thunkMiddleware from 'redux-thunk';

import pathsReducer from './pathsSlice';
import sidePanelVisibleReducer from './sidePanelVisibleSlice';
import queryReducer from './querySlice';
import isEditModeReducer from './isEditModeSlice';
import errorResultsReducer from './errorResultsSlice';
import selectedTabPathReducer from './selectedTabPathSlice';
import isFullscreenReducer from './isFullscreenSlice';

export const store = configureStore({
  reducer: {
    sidePanelVisible: sidePanelVisibleReducer,
    paths: pathsReducer,
    query: queryReducer,
    isEditMode: isEditModeReducer,
    isFullscreen: isFullscreenReducer,
    errorResults: errorResultsReducer,
    selectedTabPath: selectedTabPathReducer
  },
  middleware: [thunkMiddleware],
});

export type AppDispatch = typeof store.dispatch;
export type RootState = ReturnType<typeof store.getState>;
export type AppThunk<ReturnType = void> = ThunkAction<
  ReturnType,
  RootState,
  unknown,
  Action<string>
>;
