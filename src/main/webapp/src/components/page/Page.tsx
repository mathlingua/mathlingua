import Mark from 'mark.js';
import { BlockComment } from '../block-comment/BlockComment';
import styles from './Page.module.css';

import * as api from '../../services/api';
import { useEffect, useRef, useState } from 'react';
import { ErrorView } from '../error-view/ErrorView';
import { TopLevelEntityGroup } from '../top-level-entity-group/TopLevelEntityGroup';
import { useAppSelector } from '../../support/hooks';
import { selectQuery } from '../../store/querySlice';

import { selectViewedPath } from '../../store/viewedPathSlice';
import { Home } from '../home/Home';

import debounce from 'lodash.debounce';

import AceEditor from 'react-ace';

import 'ace-builds/src-noconflict/mode-yaml';
import 'ace-builds/src-noconflict/theme-eclipse';
import { selectIsEditMode } from '../../store/isEditModeSlice';

interface Annotation {
  row: number;
  column: number;
  text: string;
  type: 'error';
}

let scheduledFunction: { (): void; cancel(): void } | null = null;

export const Page = () => {
  const relativePath = useAppSelector(selectViewedPath) || '';
  const [fileResult, setFileResult] = useState(
    undefined as api.FileResult | undefined
  );
  const [error, setError] = useState('');
  const query = useAppSelector(selectQuery);
  const [isLoading, setIsLoading] = useState(true);
  const ref = useRef(null);

  const [annotations, setAnnotations] = useState([] as Annotation[]);
  const [editorContent, setEditorContent] = useState('');

  const isEditMode = useAppSelector(selectIsEditMode);

  const checkForErrors = async (relativePath: string) => {
    const res = await api.check();
    const errors = res.errors.filter((err) => err.path === relativePath);
    if (errors.length === 0) {
      setAnnotations([]);
    } else {
      setAnnotations(
        errors.map((err) => {
          return {
            row: err.row,
            column: err.column,
            text: err.message,
            type: 'error',
          };
        })
      );
    }
  };

  useEffect(() => {
    if (relativePath !== '') {
      api
        .getFileResult(relativePath)
        .then((fileResult) => {
          setFileResult(fileResult);
          setEditorContent(fileResult?.content ?? '');
          if (fileResult) {
            checkForErrors(fileResult.relativePath);
          }
        })
        .catch((err) => setError(err.message))
        .finally(() => setIsLoading(false));
    }
  }, [relativePath]);

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
      <div style={{ paddingLeft: '1.25em', fontSize: '120%' }}>
        {fileResult?.errors.map((error) => (
          <div style={{ color: 'darkred' }}>
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
      await checkForErrors(fileResult?.relativePath);
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
    <div className={styles.mathlinguaPage} style={{ width: '95%', padding: 0 }}>
      <div
        style={{
          display: 'flex',
        }}
      >
        <div
          style={{
            flex: '1 1 0px',
          }}
        >
          {editorView}
        </div>
        <div
          style={{
            flex: '1 1 0px',
            maxHeight: '90vh',
            overflow: 'auto',
            borderLeft: 'solid',
            borderColor: '#eeeeee',
            borderWidth: '1px',
            paddingTop: '1.5em',
          }}
        >
          {renderedContent}
        </div>
      </div>
    </div>
  );

  const pageView = (
    <div className={styles.mathlinguaPage}>{renderedContent}</div>
  );

  const contentView = isEditMode ? sideBySideView : pageView;
  return <div ref={ref}>{error ? errorView : contentView}</div>;
};
