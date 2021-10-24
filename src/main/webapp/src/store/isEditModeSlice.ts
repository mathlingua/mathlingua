import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { isStatic } from '../services/api';
import { RootState } from './store';

export interface IsEditModeState {
  isEditMode: boolean;
}

const initialState: IsEditModeState = {
  isEditMode: !isStatic(),
};

const isEditModeSlice = createSlice({
  name: 'isEditMode',
  initialState,
  reducers: {
    isEditModeUpdated(state: IsEditModeState, action: PayloadAction<boolean>) {
      state.isEditMode = action.payload;
    },
  },
});

export const selectIsEditMode = (state: RootState) =>
  state.isEditMode.isEditMode;

export const { isEditModeUpdated } = isEditModeSlice.actions;

export default isEditModeSlice.reducer;
