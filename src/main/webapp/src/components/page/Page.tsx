import Mark from 'mark.js';
import { BlockComment } from '../block-comment/BlockComment';
import styles from './Page.module.css';

import * as api from '../../services/api';
import { memo, useEffect, useRef, useState } from 'react';
import { ErrorView } from '../error-view/ErrorView';
import { TopLevelEntityGroup } from '../top-level-entity-group/TopLevelEntityGroup';
import { useAppSelector } from '../../support/hooks';
import { selectQuery } from '../../store/querySlice';

import debounce from 'lodash.debounce';

import AceEditor from 'react-ace';

import * as langTools from 'ace-builds/src-noconflict/ext-language_tools';
import 'ace-builds/src-noconflict/mode-yaml';
import 'ace-builds/src-noconflict/theme-eclipse';
import { selectIsEditMode } from '../../store/isEditModeSlice';
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
    value: 'Axiom:\ngiven?:\nif?:\niff?:\nthen:\nusing?:\nMetadata?:',
  },
  {
    name: 'Conjecture',
    value: 'Conjecture:\ngiven?:\nif?:\niff?:\nthen:\nusing?:\nMetadata?:',
  },
  {
    name: 'Theorem',
    value:
      'Theorem:\ngiven?:\nif?:\niff?:\nthen:\nusing?:\nProof?:\nMetadata?:',
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
  sidePanelWidth: number;
}

export const Page = (props: PageProps) => {
  const [fileResult, setFileResult] = useState(
    undefined as api.FileResult | undefined
  );
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const isEditMode = useAppSelector(selectIsEditMode);
  const isSidePanelVisible = useAppSelector(selectSidePanelVisible);
  const ref = useRef(null);

  useEffect(() => {
    api.getFileResult(props.viewedPath).then((fileResult) => {
      if (fileResult) {
        setFileResult(fileResult);
      } else {
        setError('Page Not Found');
      }
      setIsLoading(false);
    });
  }, [isEditMode, props.viewedPath]);

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

  const contentView = isEditMode ? (
    <SideBySideView
      relativePath={props.viewedPath}
      errors={fileResult.errors}
      entities={fileResult.entities}
      targetId={props.targetId}
    ></SideBySideView>
  ) : (
    <PageWithNavigationView
      isOnMobile={isOnMobile()}
      sidePanelWidth={props.sidePanelWidth}
      isSidePanelVisible={isSidePanelVisible}
      fileResult={fileResult}
      targetId={props.targetId}
    ></PageWithNavigationView>
  );
  return (
    <div
      style={{
        paddingLeft: !isOnMobile() ? props.sidePanelWidth : 0,
      }}
    >
      {error ? errorView : contentView}
    </div>
  );
};

const PageWithNavigationView = (props: {
  isOnMobile: boolean;
  sidePanelWidth: number;
  isSidePanelVisible: boolean;
  fileResult: api.FileResult;
  targetId: string;
}) => {
  const ref = useRef(null);
  let marginLeft: string | number = props.isOnMobile ? 0 : 'auto';
  if (ref.current && !props.isOnMobile) {
    const sideWidth = props.isSidePanelVisible ? props.sidePanelWidth : 0;
    const windowWidth = window.innerWidth;
    const thisWidth = (ref.current as any).clientWidth;
    const amountPerSide = (windowWidth - thisWidth) / 2;
    marginLeft = amountPerSide - sideWidth;
  }
  return (
    <div ref={ref} className={styles.mathlinguaPage} style={{ marginLeft }}>
      <RenderedContent
        relativePath={props.fileResult.relativePath}
        errors={props.fileResult.errors}
        entities={props.fileResult.entities}
        targetId={props.targetId}
      />
      <div className={styles.linkPanel}>
        {props.fileResult.previousRelativePath ? (
          <a
            className={styles.previousLink}
            href={`#/${props.fileResult.previousRelativePath}`}
          >
            <FontAwesomeIcon icon={faAngleLeft} />
          </a>
        ) : null}
        {props.fileResult.nextRelativePath ? (
          <a
            className={styles.nextLink}
            href={`#/${props.fileResult.nextRelativePath}`}
          >
            <FontAwesomeIcon icon={faAngleRight} />
          </a>
        ) : null}
      </div>
    </div>
  );
};

const RenderedContent = (props: {
  relativePath: string;
  errors: api.ErrorResult[];
  entities: api.EntityResult[];
  targetId: string;
}) => {
  const query = useAppSelector(selectQuery);
  const ref = useRef(null);

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
  }, [props.relativePath, props.entities, query]);

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
  }, [props.targetId, props.relativePath, query]);

  return (
    <div ref={ref}>
      <div className={styles.errorView}>
        {(props.errors || []).map((error) => (
          <div className={styles.errorItem}>
            {`ERROR (${error.row + 1}, ${error.column + 1}):`}
            <pre>{error.message}</pre>
          </div>
        ))}
      </div>
      {props.entities.map((entityResult) => (
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
                relativePath={props.relativePath}
              />
            )}
          </div>
        </span>
      ))}
    </div>
  );
};

const EditorView = memo(
  (props: {
    viewedPath: string;
    onFileResultChanged(fileResult: api.FileResult | undefined): void;
  }) => {
    const aceEditorRef = useRef(null);
    const [annotations, setAnnotations] = useState([] as Annotation[]);

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
      const editorRefVal = aceEditorRef.current as any;
      if (editorRefVal) {
        props.onFileResultChanged({
          content: '',
          entities: [],
          errors: [],
          relativePath: '',
        });
        api.readPage(props.viewedPath).then((content) => {
          editorRefVal.editor.setValue(content);
          editorRefVal.editor.clearSelection();
          editorRefVal.editor.getSession().setAnnotations(annotations);
        });
      }
    }, [props.viewedPath]);

    const saveContent = async (relativePath: string) => {
      const content = (aceEditorRef.current as any)?.editor?.getValue();
      if (content) {
        await api.writeFileResult(relativePath, content);
        return true;
      }
      return false;
    };

    const onChange = async () => {
      // only do a save/check after a short pause after the user
      // has stopped typing
      if (scheduledFunction) {
        scheduledFunction.cancel();
      }
      scheduledFunction = debounce(async () => {
        const saved = await saveContent(props.viewedPath);
        if (saved) {
          await Promise.all([
            api.check().then((resp) => {
              setAnnotations(
                resp.errors
                  .filter((err) => err.path === props.viewedPath)
                  .map((err) => ({
                    row: Math.max(0, err.row),
                    column: Math.max(0, err.column),
                    text: err.message,
                    type: 'error',
                  }))
              );
            }),
            api
              .getFileResult(props.viewedPath)
              .then((fileResult) => props.onFileResultChanged(fileResult)),
          ]);
        }
      }, 500);
      scheduledFunction();
    };

    return (
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
        enableLiveAutocompletion={false}
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
  }
);

const SideBySideView = (props: {
  relativePath: string;
  errors: api.ErrorResult[];
  entities: api.EntityResult[];
  targetId: string;
}) => {
  const [relativePath, setRelativePath] = useState(props.relativePath);
  const [errors, setErrors] = useState(props.errors);
  const [entities, setEntities] = useState(props.entities);

  useEffect(() => {
    document.body.style.setProperty(
      '--inner-height',
      `${window.innerHeight - 5}px`
    );
  }, []);

  const onFileResultChanged = (result: api.FileResult) => {
    if (result) {
      setRelativePath(result.relativePath);
      setErrors(result.errors);
      setEntities(result.entities);
    }
  };

  return (
    <div className={styles.sideBySideView}>
      <div className={styles.splitViewContainer}>
        <div className={styles.splitViewEditor}>
          <EditorView
            viewedPath={props.relativePath}
            onFileResultChanged={onFileResultChanged}
          />
        </div>
        <div className={styles.splitViewRendered}>
          <RenderedContent
            relativePath={relativePath}
            errors={errors}
            entities={entities}
            targetId={props.targetId}
          />
        </div>
      </div>
    </div>
  );
};
