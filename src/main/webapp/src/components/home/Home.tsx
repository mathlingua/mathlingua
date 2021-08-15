import { useEffect, useState } from 'react';
import RenderedComponent from '../rendered-component/RenderedComponent';
import { ErrorView } from '../error-view/ErrorView';
import styles from './Home.module.css';

import * as api from '../../services/api';
export const Home = () => {
  const [error, setError] = useState('');
  const [homeHtml, setHomeHtml] = useState('');

  useEffect(() => {
    api
      .getHomeHtml()
      .then((html) => {
        setHomeHtml(html);
      })
      .catch((err) => setError(err.message));
  }, []);

  if (!homeHtml) {
    return null;
  }

  return (
    <div className={styles.mathlinguaHome} data-testid="home">
      {error ? (
        <ErrorView message={error} />
      ) : (
        <RenderedComponent html={homeHtml} />
      )}
    </div>
  );
};
