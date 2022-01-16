import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { RootState } from './store';

export interface SelectedTabPathState {
    path: string | undefined;
}

const initialState: SelectedTabPathState = {
    path: undefined
};

const selectedTabPathSlice = createSlice({
  name: 'query',
  initialState,
  reducers: {
    selectedTabPathUpdated(state: SelectedTabPathState, action: PayloadAction<string>) {
      state.path = action.payload;
    },
  },
});

export const selectSelectedTabPath = (state: RootState) => state.selectedTabPath.path;

export const { selectedTabPathUpdated } = selectedTabPathSlice.actions;

export default selectedTabPathSlice.reducer;
