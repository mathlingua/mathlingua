import Mark from 'mark.js';
import { BlockComment } from '../block-comment/BlockComment';
import styles from './Page.module.css';

import * as api from '../../services/api';
import React, { Dispatch, useEffect, useRef, useState } from 'react';
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
import { isOnMobile } from '../../support/util';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faAngleRight, faAngleLeft } from '@fortawesome/free-solid-svg-icons';
import { DebouncedFunc } from 'lodash';
import { errorResultsUpdated } from '../../store/errorResultsSlice';
import { AppDispatch } from '../../store/store';

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
  { name: 'as', value: 'as:\nvia:\nby?:' },
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

export interface PageProps {
  viewedPath: string;
  targetId: string;
}

// needed to allow Command+s on Mac and Ctrl+s on other systems to
// save the current document
document.addEventListener('keydown', (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
  }
});

export const Page = (props: PageProps) => {
  const [fileResult, setFileResult] = useState(
    undefined as api.FileResult | undefined
  );
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(true);
  const isEditMode = useAppSelector(selectIsEditMode);

  useEffect(() => {
    api.getFileResult(props.viewedPath).then((fileResult) => {
      if (fileResult) {
        setError('');
        setFileResult(fileResult);
      } else {
        setError('Page Not Found');
      }
      setIsLoading(false);
    });
  }, [isEditMode, props.viewedPath]);

  if (error) {
    return (
      <div className={styles.mathlinguaPage}>
        <ErrorView message={error} />
      </div>
    );
  }

  if (!fileResult && !isLoading) {
    return (
      <div>
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

  return isEditMode ? (
    <SideBySideView
      relativePath={props.viewedPath}
      errors={fileResult.errors}
      entities={fileResult.entities}
      targetId={props.targetId}
    ></SideBySideView>
  ) : (
    <PageWithNavigationView
      isOnMobile={isOnMobile()}
      fileResult={fileResult}
      targetId={props.targetId}
    ></PageWithNavigationView>
  );
};

const PageWithNavigationView = (props: {
  isOnMobile: boolean;
  fileResult: api.FileResult;
  targetId: string;
}) => {
  return (
    <div className={styles.mathlinguaPage}>
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
        <div key={entityResult.id}>
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
      ))}
    </div>
  );
};

interface EditorViewProps {
  viewedPath: string;
  onFileResultChanged(fileResult: api.FileResult | undefined): void;
  dispatch: AppDispatch;
}

interface EditorViewState {
  annotations: Annotation[];
}

class EditorView extends React.Component<EditorViewProps, EditorViewState> {
  private scheduledFunction: DebouncedFunc<any> | null = null;
  private editor: any;

  constructor(props: EditorViewProps) {
    super(props);
    this.state = {
      annotations: [],
    };
  }

  async componentDidMount() {
    this.setupCompletions();
    await this.initializeEditor();
  }

  shouldComponentUpdate(
    nextProps: EditorViewProps,
    nextState: EditorViewState
  ) {
    return (
      nextProps.viewedPath !== this.props.viewedPath ||
      nextState.annotations !== this.state.annotations
    );
  }

  async componentDidUpdate(prevProps: EditorViewProps) {
    if (prevProps.viewedPath !== this.props.viewedPath) {
      if (this.scheduledFunction) {
        this.scheduledFunction.flush();
      }
      await this.initializeEditor();
    }
  }

  componentWillUnmount() {
    if (this.scheduledFunction) {
      this.scheduledFunction.cancel();
    }
  }

  private areAnnotationsEqual(ann1: Annotation, ann2: Annotation): boolean {
    return (
      ann1.column === ann2.column &&
      ann1.row === ann2.row &&
      ann1.text === ann2.text &&
      ann1.type === ann2.type
    );
  }

  private check(viewedPath: string) {
    return api.check().then((resp) => {
      const newAnnotations = resp.errors
        .filter((err) => err.path === viewedPath)
        .map(
          (err) =>
            ({
              row: Math.max(0, err.row),
              column: Math.max(0, err.column),
              text: err.message,
              type: 'error',
            } as Annotation)
        );

      let annotationsDiffer = false;
      if (this.state.annotations.length !== newAnnotations.length) {
        annotationsDiffer = true;
      } else {
        for (let i = 0; i < this.state.annotations.length; i++) {
          const thisAn = this.state.annotations[i];
          const newAn = newAnnotations[i];
          if (!this.areAnnotationsEqual(thisAn, newAn)) {
            annotationsDiffer = true;
            break;
          }
        }
      }

      if (annotationsDiffer) {
        this.setState({ ...this.state, annotations: newAnnotations });
      }
    });
  }

  private async saveAndUpdateAnnotations(viewedPath: string, content: string) {
    await api.writeFileResult(viewedPath, content);
    await Promise.all([
      this.check(viewedPath),
      api
        .getFileResult(viewedPath)
        .then((fileResult) => this.props.onFileResultChanged(fileResult)),
    ]);
  }

  private async initializeEditor() {
    this.props.onFileResultChanged({
      content: '',
      entities: [],
      errors: [],
      relativePath: '',
    });
    await api.readPage(this.props.viewedPath).then((content) => {
      this.editor.setValue(content);
      this.editor.clearSelection();
    });
    const res = await api.check();
    this.props.dispatch(
      errorResultsUpdated(
        res.errors.map((err) => ({
          row: err.row,
          column: err.column,
          message: err.message,
          relativePath: err.path,
        }))
      )
    );
  }

  private setupCompletions() {
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
  }

  render() {
    return (
      <div>
        <AceEditor
          ref={(ref) => {
            const newEditor = ref?.editor;
            if (newEditor && newEditor !== this.editor) {
              this.editor = newEditor;
            }
          }}
          mode="yaml"
          theme="eclipse"
          onChange={() => {
            if (this.scheduledFunction) {
              this.scheduledFunction.cancel();
            }
            const viewedPath = this.props.viewedPath;
            const content = this.editor.getValue();
            this.scheduledFunction = debounce(async () => {
              if (this.editor) {
                await this.saveAndUpdateAnnotations(viewedPath, content);
              }
            }, 500);
            this.scheduledFunction();
          }}
          name="ace-editor"
          editorProps={{ $blockScrolling: true }}
          highlightActiveLine={false}
          showPrintMargin={false}
          enableBasicAutocompletion={true}
          enableLiveAutocompletion={false}
          fontSize="100%"
          style={{
            fontFamily: "'Inconsolata', monospace",
            position: 'relative',
            width: '100%',
            height: 'calc(100vh - 1.75em)',
            minHeight: 'calc(100vh - 1.75em)',
            borderRight: 'solid',
            borderRightWidth: '1px',
            borderRightColor: '#dddddd',
          }}
          commands={[
            {
              name: 'save',
              bindKey: {
                win: 'Ctrl-s',
                mac: 'Command-s',
              },
              exec: () => {
                if (this.editor) {
                  return this.saveAndUpdateAnnotations(
                    this.props.viewedPath,
                    this.editor.getValue()
                  );
                }
              },
            },
          ]}
          annotations={this.state.annotations}
        ></AceEditor>
      </div>
    );
  }
}

const SideBySideView = (props: {
  relativePath: string;
  errors: api.ErrorResult[];
  entities: api.EntityResult[];
  targetId: string;
}) => {
  const [relativePath, setRelativePath] = useState(props.relativePath);
  const [errors, setErrors] = useState(props.errors);
  const [entities, setEntities] = useState(props.entities);
  const dispatch = useAppDispatch();

  const onFileResultChanged = (result: api.FileResult) => {
    if (result) {
      setRelativePath(result.relativePath);
      setErrors(result.errors);
      setEntities(result.entities);
    }
  };

  return (
    <div
      style={{ display: 'flex', flexDirection: 'row', background: '#ffffff' }}
    >
      <div style={{ width: '50%' }}>
        <EditorView
          dispatch={dispatch}
          viewedPath={props.relativePath}
          onFileResultChanged={onFileResultChanged}
        />
      </div>
      <div
        style={{
          width: '50%',
          maxHeight: 'calc(100vh - 1.75em)',
          height: 'max-content',
          overflow: 'scroll',
          background: '#ffffff',
        }}
      >
        <RenderedContent
          relativePath={relativePath}
          errors={errors}
          entities={entities}
          targetId={props.targetId}
        />
      </div>
    </div>
  );
};
