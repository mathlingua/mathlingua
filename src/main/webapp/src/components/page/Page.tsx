import Mark from 'mark.js';
import { BlockComment } from '../block-comment/BlockComment';
import styles from './Page.module.css';

import * as api from '../../services/api';
import { useEffect, useRef, useState } from 'react';
import { ErrorView } from '../error-view/ErrorView';
import { TopLevelEntityGroup } from '../top-level-entity-group/TopLevelEntityGroup';
import { useAppSelector } from '../../support/hooks';
import { selectQuery } from '../../store/querySlice';

import { useParams } from 'react-router-dom';

export const Page = () => {
  const params = (useParams() as { 0: string })[0];
  const relativePath = params;

  const [fileResult, setFileResult] = useState(
    undefined as api.FileResult | undefined
  );
  const [error, setError] = useState('');
  const query = useAppSelector(selectQuery);
  const [isLoading, setIsLoading] = useState(true);
  const ref = useRef(null);

  useEffect(() => {
    api
      .getFileResult(relativePath)
      .then((fileResult) => setFileResult(fileResult))
      .catch((err) => setError(err.message))
      .finally(() => setIsLoading(false));
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

  if (!fileResult || fileResult.entities.length === 0) {
    return null;
  }

  const pageView = (
    <div className={styles.mathlinguaPage}>
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

  return <div ref={ref}>{error ? errorView : pageView}</div>;
};
