import styles from './SidePanel.module.css';

import { useEffect, useRef, useState } from 'react';
import { pathsUpdated, selectPaths } from '../../store/pathsSlice';
import * as api from '../../services/api';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import { selectSidePanelVisible } from '../../store/sidePanelVisibleSlice';
import { ErrorView } from '../error-view/ErrorView';
import { isOnMobile } from '../../support/util';
import { PathTreeItem, PathTreeNode } from '../path-tree-item/PathTreeItem';
import { selectIsEditMode } from '../../store/isEditModeSlice';

import ResizeObserver from 'resize-observer-polyfill';

export interface SidePanelProps {
  viewedPath: string;
  onWidthChange(width: number): void;
}

export const SidePanel = (props: SidePanelProps) => {
  const dispatch = useAppDispatch();
  const visible = useAppSelector(selectSidePanelVisible);
  const paths = useAppSelector(selectPaths);
  const isEditMode = useAppSelector(selectIsEditMode);
  const divRef = useRef(null);

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

  useEffect(() => {
    if (divRef != null && divRef.current != null) {
      const observer = new ResizeObserver((entries) => {
        for (const entry of entries) {
          props.onWidthChange(entry.contentRect.width);
        }
      });
      observer.observe(divRef.current!);
    }
  }, [divRef]);

  const errorView = (
    <div ref={divRef} className={styles.sidePanel}>
      <ErrorView message={error} />
    </div>
  );

  const style = visible
    ? isOnMobile()
      ? {
          width: '100%',
          minWidth: '100%',
        }
      : {
          width: 'max-content',
          minWidth: '15em',
        }
    : {
        width: '0',
        minWidth: '0',
      };

  // if in editing mode show the 'content' directory
  const start = isEditMode ? pathData : pathData.children[0];

  const sidePanel = (
    <div ref={divRef} style={style} className={styles.sidePanel}>
      <div className={styles.sidePanelContent}>
        {start?.children?.map((node) => (
          <PathTreeItem
            key={node.name}
            node={node}
            viewedPath={props.viewedPath}
          />
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
