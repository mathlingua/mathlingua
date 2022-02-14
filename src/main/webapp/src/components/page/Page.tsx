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
import 'ace-builds/src-min-noconflict/ext-searchbox';

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

interface AceCompletion {
  value: string;
  caption: string;
  score?: number;
}

let BASE_COMPLETIONS: api.CompletionItem[] = [];
if (!api.isStatic()) {
  // the static version of the site doesn't allow editing and doesn't have a backend running
  api.getCompletions().then(completions => { BASE_COMPLETIONS = completions.items }).catch(console.error);
}

interface Annotation {
  row: number;
  column: number;
  text: string;
  type: 'error';
}

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
      this.props.dispatch(
        errorResultsUpdated(
          resp.errors.map((err) => ({
            row: err.row,
            column: err.column,
            message: err.message,
            relativePath: err.path,
          }))
        )
      );

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
    await this.check(this.props.viewedPath);

    const gutterElements = document.getElementsByClassName('ace_gutter');
    if (gutterElements) {
      for (const el of Array.from(gutterElements)) {
        (el as any).style.backgroundColor = '#f5f5f8';
        (el as any).style.borderRight = 'solid';
        (el as any).style.borderRightWidth = '1px';
        (el as any).style.borderRightColor = '#dddddd';
      }
    }

    const contentElements = document.getElementsByClassName('ace_content');
    if (contentElements) {
      for (const el of Array.from(contentElements)) {
        (el as any).style.borderTop = 'solid';
        (el as any).style.borderTopColor = '#dddddd';
        (el as any).style.borderTopWidth = '1px';
      }
    }
  }

  private getEssentialIndent(line: string) {
    let i = 0;
    while (i < line.length && (line[i] === ' ' || line[i] === '.')) {
      i++;
    }
    return i;
  }

  private setupCompletions() {
    langTools.setCompleters();
    langTools.addCompleter({
      getCompletions: async (
        editor: any,
        session: {},
        pos: { row: number; column: number },
        prefix: string,
        callback: (n: null, arr: any[]) => void
      ) => {
        const curIndent = this.getEssentialIndent(editor.session.getLine(pos.row));
        let sections: string[] = [];
        let prevRow = pos.row - 1;
        while (prevRow >= 0) {
          const tmpLine: string = editor.session.getLine(prevRow);
          if (tmpLine.trim().length === 0 || tmpLine.trim().startsWith('[')) {
            break;
          }
          const indent = this.getEssentialIndent(tmpLine);
          if (indent === curIndent) {
            sections.unshift(tmpLine.substring(curIndent).replace('?:', ':').replace(/:.*/, ':'));
          } else if (curIndent < indent) {
            break;
          }
          prevRow--;
        }

        const remoteCompletions = (
          await api.getSignatureSuffixes(`\\${prefix}`)
        ).map((suffix) => ({
          value: prefix + suffix,
          caption: prefix + suffix,
        }));

        let completions: AceCompletion[] = [];
        let completionIndex = 1;

        if (sections.length > 0) {
          for (const comp of BASE_COMPLETIONS) {
            let allMatching = true;
            let j = 0;
            for (let i=0; i<sections.length; i++) {
              let found = false;
              while (j < comp.parts.length) {
                const normalizedPart = comp.parts[j++]?.replace('[]\n', '').replace('?:', ':');
                if (sections[i] === normalizedPart) {
                  found = true;
                  break;
                }
              }

              if (!found) {
                allMatching = false;
                break;
              }
            }

            if (allMatching) {
              const firstSection = sections[0]?.replace('[]\n', '').replace('?:', ':');
              if (firstSection) {
                const index = comp.parts.map(p =>
                  p.replace('[]\n', '').replace('?:', ':')).indexOf(firstSection);
                if (index >= 0) {
                  for (let k=0; k<index; k++) {
                    if (comp.parts[k].indexOf('?') === -1) {
                      // there is a non-optional section
                      allMatching = false;
                      break;
                    }
                  }
                }
              }
            }

            if (allMatching) {
              for (let i = 0; i < comp.parts.length; i++) {
                const target = comp.parts[i]?.replace('[]\n', '')?.replace('?:', ':');
                if (target === sections[sections.length - 1]) {
                  for (let j=i+1; j<comp.parts.length; j++) {
                    const partWithoutQuestion = comp.parts[j]?.replace('[]\n', '')?.replace('?:', ':');
                    completions.push({
                      value: partWithoutQuestion,
                      caption: (comp.parts[j].indexOf('?:') >= 0 ?
                        `${partWithoutQuestion} (optional)` :
                        comp.parts[j]),
                      score: 1.0/(completionIndex++),
                    });
                  }
                  break;
                }
              }
            }
          }
        }

        if (prefix) {
          for (const comp of BASE_COMPLETIONS) {
            const first = comp.parts[0]?.replace('[]\n', '')?.replace('?:', ':');
            if (first && first.startsWith(prefix)) {
              const partWithoutQuestion = comp.parts[0]?.replace('[]\n', '')?.replace('?:', ':');
              completions.push({
                value: comp.parts[0],
                caption: (comp.parts[0].indexOf('?:') >= 0 ?
                        `${partWithoutQuestion} (optional)` :
                        comp.parts[0]),
                score: 1.0/(completionIndex++),
              });
            }
          }
        } else if (sections.length === 0) {
          completions = completions.concat(BASE_COMPLETIONS.map(comp => {
            const partWithoutQuestion = comp.parts[0]?.replace('[]\n', '')?.replace('?:', ':');
            return {
              caption: (comp.parts[0].indexOf('?:') >= 0 ?
                        `${partWithoutQuestion} (optional)` :
                        comp.parts[0]),
              value: comp.parts[0],
            };
          }));
        }

        if (prefix !== '') {
          completions = completions.concat(remoteCompletions);
        }

        callback(null, completions);
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
          tabSize={2}
          setOptions={{
            useSoftTabs: true,
          }}
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
            borderLeft: 'solid',
            borderLeftWidth: '1px',
            borderLeftColor: '#aaaaaa',
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
          borderTop: 'solid',
          borderTopColor: '#dddddd',
          borderTopWidth: '1px'
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
