import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { RootState } from './store';

export interface QueryState {
  query: string;
}

const initialState: QueryState = {
  query: '',
};

const querySlice = createSlice({
  name: 'query',
  initialState,
  reducers: {
    queryUpdated(state: QueryState, action: PayloadAction<string>) {
      state.query = action.payload;
    },
  },
});

export const selectQuery = (state: RootState) => state.query.query;

export const { queryUpdated } = querySlice.actions;

export default querySlice.reducer;
