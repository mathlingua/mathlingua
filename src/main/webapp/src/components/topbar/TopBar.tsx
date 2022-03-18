import styles from './TopBar.module.css';

import { SidePanelButton } from '../side-panel-button/SidePanelButton';
import { SearchPanel } from '../search-panel/SearchPanel';
import { GitHubIcon } from '../github-icon/GitHubIcon';
import { isStatic } from '../../services/api';
import { EditModeButton } from '../edit-mode-button/EditModeButton';
import { HelpButton } from '../help-button/HelpButton';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import { editorFontUpdated, selectEditorFont } from '../../store/editorFontSlice';
import { editorFontSizeUpdated, selectEditorFontSize } from '../../store/editorFontSizeSlice';
import { selectIsEditMode } from '../../store/isEditModeSlice';

export const TopBar = () => {
  const dispatch = useAppDispatch();
  const font = useAppSelector(selectEditorFont);
  const fontSize = useAppSelector(selectEditorFontSize);
  const isEditMode = useAppSelector(selectIsEditMode);

  const editorControls = (
    <>
      <select style={{
          border: 'solid',
          borderWidth: '1px',
          borderRadius: '1px',
          borderColor: '#cccccc',
          background: 'white',
          marginRight: '1em',
          paddingLeft: '1ex'
        }}
        defaultValue={font}
        onClick={(event: any) => {
          const font = event.target?.value;
          if (font) {
            window.localStorage.setItem('mlg.editorFont', font);
            dispatch(editorFontUpdated(font));
          }
        }}>
        <option value='Anonymous Pro'>Anonymous Pro</option>
        <option value='CMU Typewriter Text'>CMU Typewriter Text</option>
        <option value='Courier Prime'>Courier Prime</option>
        <option value='Cousine'>Cousine</option>
        <option value='Cutive Mono'>Cutive Mono</option>
        <option value='Fira Code'>Fira Code</option>
        <option value='Fira Mono'>Fira Mono</option>
        <option value='Inconsolata'>Inconsolata</option>
        <option value='Libertinus Mono'>Libertinus Mono</option>
        <option value='Nova Mono'>Nova Mono</option>
        <option value='Noto Sans Mono'>Noto Sans Mono</option>
        <option value='PT Mono'>PT Mono</option>
        <option value='Red Hat Mono'>Red Hat Mono</option>
        <option value='Roboto Mono'>Roboto Mono</option>
        <option value='Share Tech Mono'>Share Tech Mono</option>
        <option value='Source Code Pro'>Source Code Pro</option>
        <option value='Space Mono'>Space Mono</option>
        <option value='Syne Mono'>Syne Mono</option>
        <option value='The Good Monolith'>The Good Monolith</option>
        <option value='Ubuntu Mono'>Ubuntu Mono</option>
      </select>
      <select style={{
          border: 'solid',
          borderWidth: '1px',
          borderRadius: '1px',
          borderColor: '#cccccc',
          background: 'white',
          marginRight: '1em',
          paddingLeft: '1ex'
        }}
        defaultValue={fontSize}
        onClick={(event: any) => {
          const fontSize = event.target?.value;
          if (fontSize) {
            window.localStorage.setItem('mlg.editorFontSize', fontSize);
            dispatch(editorFontSizeUpdated(fontSize));
          }
        }}>
        <option value='8'>8</option>
        <option value='9'>9</option>
        <option value='10'>10</option>
        <option value='11'>11</option>
        <option value='12'>12</option>
        <option value='13'>13</option>
        <option value='14'>14</option>
        <option value='15'>15</option>
        <option value='16'>16</option>
        <option value='17'>17</option>
        <option value='18'>18</option>
        <option value='19'>19</option>
        <option value='20'>20</option>
        <option value='21'>21</option>
        <option value='22'>22</option>
        <option value='23'>23</option>
        <option value='24'>24</option>
        <option value='25'>25</option>
        <option value='26'>26</option>
        <option value='27'>27</option>
        <option value='28'>28</option>
        <option value='29'>29</option>
        <option value='30'>30</option>
      </select>
    </>
  );

  return (
    <div className={styles.topbar}>
      <SidePanelButton />
      <SearchPanel />
      {isEditMode ? editorControls : null}
      {isStatic() ? null : <EditModeButton />}
      <GitHubIcon />
      <HelpButton />
    </div>
  );
};
