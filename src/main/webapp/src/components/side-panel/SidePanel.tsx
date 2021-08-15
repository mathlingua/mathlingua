import styles from './SidePanel.module.css';

import { useEffect, useState } from 'react';
import { pathsUpdated, selectPaths } from '../../store/pathsSlice';
import * as api from '../../services/api';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import { selectSidePanelVisible } from '../../store/sidePanelVisibleSlice';
import { ErrorView } from '../error-view/ErrorView';
import { isOnMobile } from '../../support/util';
import { Link } from 'react-router-dom';
import { useLocation } from 'react-router-dom';
import { PathTreeItem, PathTreeNode } from '../path-tree-item/PathTreeItem';

export const SidePanel = () => {
  const dispatch = useAppDispatch();
  const visible = useAppSelector(selectSidePanelVisible);
  const paths = useAppSelector(selectPaths);

  const location = useLocation();
  const viewedPath = location.pathname.substring(1);

  const [allPaths, setAllPaths] = useState([] as string[]);
  const [error, setError] = useState('');
  const [pathData, setPathData] = useState({
    name: '',
    isDir: true,
    path: undefined,
    children: [],
  } as PathTreeNode);

  useEffect(() => {
    api
      .getAllPaths()
      .then((paths) => {
        setAllPaths(paths);
        dispatch(pathsUpdated(paths));
      })
      .catch((err) => setError(err.message));
  }, [dispatch]);

  useEffect(() => {
    const pathsToView = paths ?? allPaths;
    setPathData(allPathsToTreeNode(pathsToView));
  }, [allPaths, paths]);

  const errorView = (
    <div className={styles.sidePanel}>
      <ErrorView message={error} />
    </div>
  );

  const style = visible
    ? isOnMobile()
      ? {
          width: '100%',
        }
      : {
          minWidth: '15em',
        }
    : {
        width: '0',
      };

  const sidePanel = (
    <div style={style} className={styles.sidePanel}>
      <div className={styles.sidePanelContent}>
        <Link
          to="/"
          key="home"
          className={
            !viewedPath
              ? `${styles.sidePanelItem} ${styles.mathlinguaHomeItem} ${styles.mathlinguaListFileItem} ${styles.selected}`
              : `${styles.sidePanelItem} ${styles.mathlinguaHomeItem} ${styles.mathlinguaListFileItem}`
          }
        >
          <div>Home</div>
        </Link>
        <hr />
        {pathData.children[0]?.children?.map((node) => (
          <PathTreeItem key={node.name} node={node} />
        ))}
      </div>
    </div>
  );

  return error ? errorView : sidePanel;
};

function allPathsToTreeNode(allPaths: string[]): PathTreeNode {
  const root: PathTreeNode = {
    name: '',
    isDir: true,
    path: undefined,
    children: [],
  };
  for (const path of allPaths) {
    populateTreeNode(root, path);
  }
  return root;
}

function populateTreeNode(root: PathTreeNode, path: string) {
  const parts = path.split('/');
  let cur = root;
  for (let i = 0; i < parts.length; i++) {
    const p = parts[i];
    const child = cur.children.find((child) => child.name === p);
    if (child) {
      cur = child;
    } else {
      const isDir = i !== parts.length - 1;
      const newChild: PathTreeNode = {
        name: p,
        isDir,
        path: isDir ? undefined : path,
        children: [],
      };
      cur.children.push(newChild);
      cur = newChild;
    }
  }
}
