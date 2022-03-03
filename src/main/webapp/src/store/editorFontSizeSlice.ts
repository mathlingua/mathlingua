import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { RootState } from './store';

export interface EditorFontSizeState {
    fontSize: string;
}

const initialState: EditorFontSizeState = {
    fontSize: window.localStorage.getItem('mlg.editorFontSize') ?? '18'
};

const editorFontSizeSlice = createSlice({
  name: 'editorFontSize',
  initialState,
  reducers: {
    editorFontSizeUpdated(state: EditorFontSizeState, action: PayloadAction<string>) {
      state.fontSize = action.payload;
    },
  },
});

export const selectEditorFontSize = (state: RootState) => state.editorFontSize.fontSize;

export const { editorFontSizeUpdated } = editorFontSizeSlice.actions;

export default editorFontSizeSlice.reducer;
