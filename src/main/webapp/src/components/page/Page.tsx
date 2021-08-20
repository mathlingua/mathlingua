import Mark from 'mark.js';
import { BlockComment } from '../block-comment/BlockComment';
import styles from './Page.module.css';

import * as api from '../../services/api';
import { useEffect, useRef, useState } from 'react';
import { ErrorView } from '../error-view/ErrorView';
import { TopLevelEntityGroup } from '../top-level-entity-group/TopLevelEntityGroup';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import { selectQuery } from '../../store/querySlice';

import { selectViewedPath } from '../../store/viewedPathSlice';
import { Home } from '../home/Home';

import debounce from 'lodash.debounce';

import AceEditor from 'react-ace';

import 'ace-builds/src-noconflict/mode-yaml';
import 'ace-builds/src-noconflict/theme-eclipse';
import { selectIsEditMode } from '../../store/isEditModeSlice';
import {
  errorResultsUpdated,
  selectErrorResults,
} from '../../store/errorResultsSlice';

interface Annotation {
  row: number;
  column: number;
  text: string;
  type: 'error';
}

let scheduledFunction: { (): void; cancel(): void } | null = null;

export const Page = () => {
  const dispatch = useAppDispatch();
  const relativePath = useAppSelector(selectViewedPath) || '';
  const [fileResult, setFileResult] = useState(
    undefined as api.FileResult | undefined
  );
  const [error, setError] = useState('');
  const query = useAppSelector(selectQuery);
  const [isLoading, setIsLoading] = useState(true);
  const ref = useRef(null);

  const [annotations, setAnnotations] = useState([] as Annotation[]);
  const errorResults = useAppSelector(selectErrorResults);
  const [editorContent, setEditorContent] = useState('');

  const isEditMode = useAppSelector(selectIsEditMode);

  const checkForErrors = async () => {
    const res = await api.check();
    dispatch(
      errorResultsUpdated(
        res.errors.map((err) => ({
          row: err.row,
          column: err.column,
          message: err.message,
          relativePath: err.path,
        }))
      )
    );
  };

  useEffect(() => {
    if (relativePath !== '') {
      api
        .getFileResult(relativePath)
        .then(async (fileResult) => {
          setFileResult(fileResult);
          setEditorContent(fileResult?.content ?? '');
          await checkForErrors();
        })
        .catch((err) => setError(err.message))
        .finally(() => setIsLoading(false));
    }
  }, [relativePath]);

  useEffect(() => {
    setAnnotations(
      errorResults
        .filter((err) => err.relativePath === relativePath)
        .map((err) => ({
          row: err.row,
          column: err.column,
          text: err.message,
          type: 'error',
        }))
    );
  }, [errorResults]);

  useEffect(() => {
    if (ref.current) {
      const markInstance = new Mark(ref.current!);
      markInstance.unmark({
        done: () => {
          if (query.length > 0) {
            markInstance.mark(query, {
              caseSensitive: false,
              separateWordSearch: true,
            });
          }
        },
      });
    }
  }, [relativePath, fileResult, query]);

  if (relativePath === '') {
    return <Home />;
  }

  const errorView = (
    <div className={styles.mathlinguaPage}>
      <ErrorView message={error} />
    </div>
  );

  if (!fileResult && !isLoading) {
    return (
      <div className={styles.mathlinguaPage}>
        <ErrorView message="Page Not Found" />
      </div>
    );
  }

  if (
    !fileResult ||
    (fileResult.entities.length === 0 && fileResult.errors.length === 0)
  ) {
    return null;
  }

  const renderedContent = (
    <div>
      <div className={styles.errorView}>
        {fileResult?.errors.map((error) => (
          <div className={styles.errorItem}>
            {`ERROR (${error.row + 1}, ${error.column + 1}):`}
            <pre>{error.message}</pre>
          </div>
        ))}
      </div>
      {fileResult?.entities.map((entityResult) => (
        <span key={entityResult.id}>
          {entityResult.type === 'TopLevelBlockComment' ? (
            <BlockComment
              renderedHtml={entityResult.renderedHtml}
              rawHtml={entityResult.rawHtml}
            />
          ) : (
            <TopLevelEntityGroup entity={entityResult} />
          )}
        </span>
      ))}
    </div>
  );

  const onChange = async (newValue: string) => {
    // update the editor content immediately
    setEditorContent(newValue);
    // but only do a save/check after short pause after the user
    // has paused typing
    if (scheduledFunction) {
      scheduledFunction.cancel();
    }
    scheduledFunction = debounce(async () => {
      await api.writeFileResult(fileResult.relativePath, newValue);
      await checkForErrors();
      const fileRes = await api.getFileResult(fileResult.relativePath);
      setFileResult(fileRes);
    }, 500);
    scheduledFunction();
  };

  const editorView = (
    <AceEditor
      mode="yaml"
      theme="eclipse"
      onChange={onChange}
      name="ace-editor"
      editorProps={{ $blockScrolling: true }}
      value={editorContent}
      highlightActiveLine={false}
      showPrintMargin={false}
      fontSize="90%"
      style={{
        width: '100%',
        height: '100%',
        minHeight: '90vh',
      }}
      annotations={annotations}
    ></AceEditor>
  );

  const sideBySideView = (
    <div className={styles.mathlinguaPage + ' ' + styles.sideBySideView}>
      <div className={styles.splitViewContainer}>
        <div className={styles.splitViewEditor}>{editorView}</div>
        <div className={styles.splitViewRendered}>{renderedContent}</div>
      </div>
    </div>
  );

  const pageView = (
    <div className={styles.mathlinguaPage}>{renderedContent}</div>
  );

  const contentView = isEditMode ? sideBySideView : pageView;
  return <div ref={ref}>{error ? errorView : contentView}</div>;
};
