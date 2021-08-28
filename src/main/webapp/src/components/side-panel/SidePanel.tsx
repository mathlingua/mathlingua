import styles from './SidePanel.module.css';

import { useEffect, useState } from 'react';
import { pathsUpdated, selectPaths } from '../../store/pathsSlice';
import * as api from '../../services/api';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import { selectSidePanelVisible } from '../../store/sidePanelVisibleSlice';
import { ErrorView } from '../error-view/ErrorView';
import { isOnMobile } from '../../support/util';
import { PathTreeItem, PathTreeNode } from '../path-tree-item/PathTreeItem';
import { selectViewedPath } from '../../store/viewedPathSlice';
import { selectIsEditMode } from '../../store/isEditModeSlice';

export const SidePanel = () => {
  const dispatch = useAppDispatch();
  const visible = useAppSelector(selectSidePanelVisible);
  const paths = useAppSelector(selectPaths);
  const isEditMode = useAppSelector(selectIsEditMode);

  const [allPaths, setAllPaths] = useState([] as string[]);
  const [error, setError] = useState('');
  const [pathData, setPathData] = useState({
    name: '',
    isDir: true,
    path: '',
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

  // if in editing mode show the 'content' directory
  const start = isEditMode ? pathData : pathData.children[0];

  const sidePanel = (
    <div style={style} className={styles.sidePanel}>
      <div className={styles.sidePanelContent}>
        {start?.children?.map((node) => (
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
    path: '',
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
        path: parts.slice(0, i + 1).join('/'),
        children: [],
      };
      cur.children.push(newChild);
      cur = newChild;
    }
  }
}
