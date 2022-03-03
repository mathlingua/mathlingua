import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { RootState } from './store';

export interface EditorFontState {
    font: string;
}

const initialState: EditorFontState = {
    font: window.localStorage.getItem('mlg.editorFont') ?? 'Inconsolata'
};

const editorFontSlice = createSlice({
  name: 'editorFont',
  initialState,
  reducers: {
    editorFontUpdated(state: EditorFontState, action: PayloadAction<string>) {
      state.font = action.payload;
    },
  },
});

export const selectEditorFont = (state: RootState) => state.editorFont.font;

export const { editorFontUpdated } = editorFontSlice.actions;

export default editorFontSlice.reducer;
