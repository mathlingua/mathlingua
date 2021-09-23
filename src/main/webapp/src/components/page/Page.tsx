import Mark from 'mark.js';
import { BlockComment } from '../block-comment/BlockComment';
import styles from './Page.module.css';

import * as api from '../../services/api';
import { useEffect, useRef, useState } from 'react';
import { ErrorView } from '../error-view/ErrorView';
import { TopLevelEntityGroup } from '../top-level-entity-group/TopLevelEntityGroup';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import { selectQuery } from '../../store/querySlice';

import debounce from 'lodash.debounce';

import AceEditor from 'react-ace';

import * as langTools from 'ace-builds/src-noconflict/ext-language_tools';
import 'ace-builds/src-noconflict/mode-yaml';
import 'ace-builds/src-noconflict/theme-eclipse';
import { selectIsEditMode } from '../../store/isEditModeSlice';
import {
  errorResultsUpdated,
  selectErrorResults,
} from '../../store/errorResultsSlice';
import { selectSidePanelVisible } from '../../store/sidePanelVisibleSlice';
import { isOnMobile } from '../../support/util';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faAngleRight, faAngleLeft } from '@fortawesome/free-solid-svg-icons';

interface Annotation {
  row: number;
  column: number;
  text: string;
  type: 'error';
}

interface Completion {
  name: string;
  value: string;
}

const BASE_COMPLETIONS: Completion[] = [
  { name: 'and', value: 'and:' },
  { name: 'exists', value: 'exists:\nsuchThat:' },
  { name: 'existsUnique', value: 'existsUnique:\nsuchThat:' },
  { name: 'forAll', value: 'forAll:\nsuchThat?:\nthen:' },
  { name: 'if', value: 'if:\nthen:' },
  { name: 'iff', value: 'iff:\nthen:' },
  { name: 'not', value: 'not:' },
  { name: 'or', value: 'or:' },
  { name: 'piecewise', value: 'piecewise:' },
  {
    name: 'Defines:',
    value:
      'Defines:\nrequiring?:\nwhen?:\nmeans:\nevaluated?:\nviewing?:\nusing?:\nwritten:\ncalled:\nMetadata?:',
  },
  {
    name: 'States',
    value:
      'States:\nrequiring?:\nwhen?:\nthat:\nusing?:\nwritten:\ncalled:\nMetadata?:',
  },
  { name: 'equality', value: 'equality:\nbetween:\nprovided:' },
  { name: 'membership', value: 'membership:\nthrough:' },
  { name: 'as', value: 'as:\nvia:' },
  {
    name: 'Resource',
    value:
      'Resource:\n. type?: ""\n. name?: ""\n. author?: ""\n. homepage?: ""\n. url?: ""\n. offset?: ""\nMetadata?:',
  },
  {
    name: 'Axiom',
    value: 'Axiom:\ngiven?:\nwhere?:\nif?:\niff?:\nthen:\nusing?:\nMetadata?:',
  },
  {
    name: 'Conjecture',
    value:
      'Conjecture:\ngiven?:\nwhere?:\nif?:\niff?:\nthen:\nusing?:\nMetadata?:',
  },
  {
    name: 'Theorem',
    value:
      'Theorem:\ngiven?:\nwhere?:\nif?:\niff?:\nthen:\nusing?:\nProof?:\nMetadata?:',
  },
  { name: 'Topic', value: 'Topic:\ncontent:\nMetadata?:' },
  { name: 'Note', value: 'Note:\ncontent:\nMetadata?:' },
];

let scheduledFunction: { (): void; cancel(): void } | null = null;

window.addEventListener('resize', () => {
  document.body.style.setProperty(
    '--inner-height',
    `${window.innerHeight - 5}px`
  );
});

export interface PageProps {
  viewedPath: string;
  targetId: string;
}

export const Page = (props: PageProps) => {
  const dispatch = useAppDispatch();
  const [fileResult, setFileResult] = useState(
    undefined as api.FileResult | undefined
  );
  const [error, setError] = useState('');
  const query = useAppSelector(selectQuery);
  const [isLoading, setIsLoading] = useState(true);
  const ref = useRef(null);
  const aceEditorRef = useRef(null);

  const [annotations, setAnnotations] = useState([] as Annotation[]);
  const errorResults = useAppSelector(selectErrorResults);
  const [editorContent, setEditorContent] = useState('');

  const isEditMode = useAppSelector(selectIsEditMode);

  const isSidePanelVisible = useAppSelector(selectSidePanelVisible);

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
    api
      .getFileResult(props.viewedPath)
      .then(async (fileResult) => {
        setFileResult(fileResult);
        setEditorContent(fileResult?.content ?? '');
        setError('');
        await checkForErrors();
      })
      .catch((err) => setError(err.message))
      .finally(() => setIsLoading(false));
  }, [props.viewedPath, props.targetId]);

  useEffect(() => {
    setAnnotations(
      errorResults
        .filter((err) => err.relativePath === props.viewedPath)
        .map((err) => ({
          row: err.row,
          column: err.column,
          text: err.message,
          type: 'error',
        }))
    );
  }, [errorResults, props.viewedPath]);

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
  }, [props.viewedPath, fileResult, query]);

  useEffect(() => {
    if (ref.current) {
      window.scroll({
        top: 0,
        left: 0,
        behavior: 'auto',
      });
      const el = document.getElementById(props.targetId);
      if (el) {
        el.scrollIntoView();
      }
    }
  }, [props.viewedPath, props.targetId, fileResult, query]);

  useEffect(() => {
    langTools.setCompleters();
    langTools.addCompleter({
      getCompletions: async (
        editor: any,
        session: {},
        pos: { column: number },
        prefix: string,
        callback: (n: null, arr: any[]) => void
      ) => {
        if (prefix.length === 0) {
          return callback(null, []);
        }
        const column = pos.column;
        let indent = '\n';
        for (let i = 0; i < column - prefix.length; i++) {
          indent += ' ';
        }
        const remoteCompletions = (
          await api.getSignatureSuffixes(`\\${prefix}`)
        ).map((suffix) => ({
          name: prefix + suffix,
          value: prefix + suffix,
        }));
        callback(
          null,
          BASE_COMPLETIONS.concat(remoteCompletions).map((item) => {
            return {
              name: item.name.replace(/\\/, ''),
              value: item.value.replace(/\\/, '').replace(/\n/g, indent),
            };
          })
        );
      },
    });
  }, []);

  useEffect(() => {
    document.body.style.setProperty(
      '--inner-height',
      `${window.innerHeight - 5}px`
    );
  }, []);

  useEffect(() => {
    const editorRefVal = aceEditorRef.current as any;
    if (editorRefVal) {
      editorRefVal.editor.setValue(editorContent);
      editorRefVal.editor.clearSelection();
    }
  }, [isEditMode, editorContent]);

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
          <div>
            {entityResult.type === 'TopLevelBlockComment' ? (
              <BlockComment
                renderedHtml={entityResult.renderedHtml}
                rawHtml={entityResult.rawHtml}
              />
            ) : (
              <TopLevelEntityGroup
                entity={entityResult}
                relativePath={props.viewedPath}
              />
            )}
          </div>
        </span>
      ))}
    </div>
  );

  async function saveContent(relativePath: string): Promise<boolean> {
    const content = (aceEditorRef.current as any)?.editor?.getValue();
    if (content) {
      await api.writeFileResult(relativePath, content);
      return true;
    }
    return false;
  }

  const onChange = async () => {
    // only do a save/check after a short pause after the user
    // has stopped typing
    if (scheduledFunction) {
      scheduledFunction.cancel();
    }
    scheduledFunction = debounce(async () => {
      const saved = await saveContent(fileResult.relativePath);
      if (saved) {
        await checkForErrors();
        const fileRes = await api.getFileResult(fileResult.relativePath);
        setFileResult(fileRes);
      }
    }, 500);
    scheduledFunction();
  };

  const editorView = (
    <AceEditor
      ref={aceEditorRef}
      mode="yaml"
      theme="eclipse"
      onChange={onChange}
      name="ace-editor"
      editorProps={{ $blockScrolling: true }}
      highlightActiveLine={false}
      showPrintMargin={false}
      enableBasicAutocompletion={true}
      enableLiveAutocompletion={true}
      fontSize="90%"
      style={{
        position: 'relative',
        width: '100%',
        height: '100%',
        minHeight: '100vh',
      }}
      annotations={annotations}
    ></AceEditor>
  );

  const sideBySideView = (
    <div className={styles.sideBySideView}>
      <div className={styles.splitViewContainer}>
        <div className={styles.splitViewEditor}>{editorView}</div>
        <div className={styles.splitViewRendered}>{renderedContent}</div>
      </div>
    </div>
  );

  const pageView = (
    <div className={styles.mathlinguaPage}>
      {renderedContent}
      <div className={styles.linkPanel}>
        {fileResult.previousRelativePath ? (
          <a
            className={styles.previousLink}
            href={`#/${fileResult.previousRelativePath}`}
          >
            <FontAwesomeIcon icon={faAngleLeft} />
          </a>
        ) : null}
        {fileResult.nextRelativePath ? (
          <a
            className={styles.nextLink}
            href={`#/${fileResult.nextRelativePath}`}
          >
            <FontAwesomeIcon icon={faAngleRight} />
          </a>
        ) : null}
      </div>
    </div>
  );

  const contentView = isEditMode ? sideBySideView : pageView;
  return (
    <div
      ref={ref}
      style={{
        transition: '0.2s',
        paddingLeft:
          isEditMode && !isOnMobile() && isSidePanelVisible ? '15em' : '0',
      }}
    >
      {error ? errorView : contentView}
    </div>
  );
};
