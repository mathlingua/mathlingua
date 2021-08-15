import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { RootState } from './store';
import { isOnMobile } from '../support/util';

export interface SidePanelVisibleState {
  sidePanelVisible: boolean;
}

const initialState: SidePanelVisibleState = {
  sidePanelVisible: !isOnMobile(),
};

const sidePanelVisibleSlice = createSlice({
  name: 'sidePanelVisible',
  initialState,
  reducers: {
    sidePanelVisibilityChanged(
      state: SidePanelVisibleState,
      action: PayloadAction<boolean>
    ) {
      state.sidePanelVisible = action.payload;
    },
  },
});

export const selectSidePanelVisible = (state: RootState) =>
  state.sidePanelVisible.sidePanelVisible;

export const { sidePanelVisibilityChanged } = sidePanelVisibleSlice.actions;

export default sidePanelVisibleSlice.reducer;
