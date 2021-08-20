import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { RootState } from './store';
import * as api from '../services/api';

export interface ErrorResultsState {
  errors: api.ErrorResult[];
}

const initialState: ErrorResultsState = {
  errors: [],
};

const errorResultsSlice = createSlice({
  name: 'errorResults',
  initialState,
  reducers: {
    errorResultsUpdated(
      state: ErrorResultsState,
      action: PayloadAction<api.ErrorResult[]>
    ) {
      state.errors = action.payload;
    },
  },
});

export const selectErrorResults = (state: RootState) =>
  state.errorResults.errors;

export const { errorResultsUpdated } = errorResultsSlice.actions;

export default errorResultsSlice.reducer;
