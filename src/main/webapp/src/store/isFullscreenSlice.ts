import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { RootState } from './store';

export interface IsFullscreenState {
  isFullscreen: boolean;
}

const initialState: IsFullscreenState = {
    isFullscreen: false
};

const isFullscreenSlice = createSlice({
  name: 'isFullscreen',
  initialState,
  reducers: {
    isFullscreenUpdated(state: IsFullscreenState, action: PayloadAction<boolean>) {
      state.isFullscreen = action.payload;
    },
  },
});

export const selectIsFullscreen = (state: RootState) =>
  state.isFullscreen.isFullscreen;

export const { isFullscreenUpdated } = isFullscreenSlice.actions;

export default isFullscreenSlice.reducer;
