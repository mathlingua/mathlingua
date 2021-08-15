import { configureStore, ThunkAction, Action } from '@reduxjs/toolkit';

import thunkMiddleware from 'redux-thunk';

import pathsReducer from './pathsSlice';
import sidePanelVisibleReducer from './sidePanelVisibleSlice';
import queryReducer from './querySlice';

export const store = configureStore({
  reducer: {
    sidePanelVisible: sidePanelVisibleReducer,
    paths: pathsReducer,
    query: queryReducer,
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
