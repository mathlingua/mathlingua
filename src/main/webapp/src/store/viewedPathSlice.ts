import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { RootState } from './store';

export interface ViewedPathState {
  path: string | undefined;
}

const initialState: ViewedPathState = {
  path: undefined,
};

const viewedPathSlice = createSlice({
  name: 'viewedFile',
  initialState,
  reducers: {
    viewedPathUpdated(
      state: ViewedPathState,
      action: PayloadAction<string | undefined>
    ) {
      state.path = action.payload;
    },
  },
});

export const selectViewedPath = (state: RootState) => state.viewedPath.path;

export const { viewedPathUpdated } = viewedPathSlice.actions;

export default viewedPathSlice.reducer;
