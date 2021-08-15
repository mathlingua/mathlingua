import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { AppThunk, RootState } from './store';
import * as api from '../services/api';

export interface PathsState {
  paths: string[] | undefined;
}

const initialState: PathsState = {
  paths: [],
};

const pathsSlice = createSlice({
  name: 'paths',
  initialState,
  reducers: {
    pathsUpdated(
      state: PathsState,
      action: PayloadAction<string[] | undefined>
    ) {
      if (!action.payload) {
        state.paths = action.payload;
      } else {
        state.paths = [...action.payload];
      }
    },
  },
});

export const selectPaths = (state: RootState) => state.paths.paths;

export const { pathsUpdated } = pathsSlice.actions;

export const updatePathsForQuery =
  (query: string): AppThunk =>
  async (dispatch) => {
    const paths = await api.search(query);
    dispatch(pathsUpdated(paths));
  };

export default pathsSlice.reducer;
