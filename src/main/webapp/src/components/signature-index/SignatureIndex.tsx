
import styles from './SignatureIndex.module.css';

import * as api from '../../services/api';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import RenderedComponent from '../rendered-component/RenderedComponent';

export const SignatureIndex = () => {
  const [index, setIndex] = useState({ entries: [] } as api.SignatureIndex);

  useEffect(() => {
    api.getSignatureIndex().then(setIndex)
  }, []);

  const calledPairs: Array<{ name: string; entry: api.SignatureIndexEntry; }> = [];
  for (const entry of index.entries) {
    for (const name of entry.called) {
      calledPairs.push({
        name,
        entry
      });
    }
  }

  function removeMathLaTeX(text: string) {
    const trimmed = text.trim();
    if (trimmed.startsWith('$') && trimmed.endsWith('$')) {
      // it is of the form `$\textbf{sometext} some math \textbf{more text}$`
      const middle = trimmed.substring(1, trimmed.length - 1);
      return middle.replace(/\\[a-zA-Z]+/g, '')
                   .replace(/\{/g, '')
                   .replace(/\}/g, '')
                   .replace(/[a-zA-Z]+\?/g, '')
                   .toUpperCase()
                   .trim();
    }

    // it is of the form `some text $some math$ more text`
    return text.replace(/\$\$.*?\$\$/g, '')
               .replace(/\$.*?\$/g, '')
               .replace(/\(.*?\)/g, '')
               .replace(/\[.*?\]/g, '')
               .replace(/"/g, '')
               .toUpperCase()
               .trim();
  }

  return <div className={styles.indexPage}>
    <h1>Index by Name</h1>
    <ul className={styles.indexList}>
    {
      calledPairs.sort((pair1, pair2) => {
        const name1 = removeMathLaTeX(pair1.name);
        const name2 = removeMathLaTeX(pair2.name);
        return name1.localeCompare(name2);
      }).map(
          (pair, index) =><li className={styles.indexListItem}
                     key={index}>
            <Link className={styles.indexListCalledLink}
                  to={`${pair.entry.relativePath}#${pair.entry.id}`}>
              <RenderedComponent html={pair.name} />
            </Link>
            <span className={styles.indexItemCalledLocation}>
              {pair.entry.relativePath
                    .replace(/\d+((\\.)?\d+)?_/g, '')
                    .replace(/_/g, ' ')
                    .replace(/\.math/g, '')
                    .replace(/content\//, '')
                    .replace(/\//g, ' ▸ ')}
            </span>
          </li>)
    }
    </ul>
    <h1>Index by Signature</h1>
    <ul className={styles.indexList}>
    {
      index.entries.filter(entry => !!entry.signature).sort((entry1, entry2) => {
        const sig1 = entry1.signature || '';
        const sig2 = entry2.signature || '';
        return sig1.localeCompare(sig2);
      }).map(
          (entry, index) =><li className={styles.indexListItem}
                               key={index}>
            <Link className={styles.indexItemLink}
                  to={`${entry.relativePath}#${entry.id}`}>
                {entry.signature}
            </Link>
            <div className={styles.indexItemLocation}>
              {entry.relativePath
                    .replace(/\d+((\\.)?\d+)?_/g, '')
                    .replace(/_/g, ' ')
                    .replace(/\.math/g, '')
                    .replace(/content\//, '')
                    .replace(/\//g, ' ▸ ')}
            </div>
          </li>)
    }
    </ul>
  </div>;
};
