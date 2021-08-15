import { useState } from 'react';
import { Link } from 'react-router-dom';
import styles from './PathTreeItem.module.css';

import { sidePanelVisibilityChanged } from '../../store/sidePanelVisibleSlice';
import { useAppDispatch } from '../../support/hooks';
import { isOnMobile } from '../../support/util';

export interface PathTreeNode {
  name: string;
  isDir: boolean;
  path?: string;
  children: PathTreeNode[];
}

export interface PathTreeItemProps {
  node: PathTreeNode;
}

export const PathTreeItem = (props: PathTreeItemProps) => {
  const dispatch = useAppDispatch();
  const [isExpanded, setIsExpanded] = useState(false);

  if (props.node.isDir) {
    return (
      <span>
        <li
          className={styles.mathlinguaListDirItem + ' ' + styles.sidePanelItem}
          onClick={() => setIsExpanded(!isExpanded)}
        >
          {isExpanded ? (
            <button className={styles.triangle}>&#9662;</button>
          ) : (
            <button className={styles.triangle}>&#9656;</button>
          )}
          {props.node.name.replace('_', ' ')}
        </li>
        {isExpanded ? (
          <ul>
            {props.node.children.map((child) => (
              <PathTreeItem key={child.name} node={child} />
            ))}
          </ul>
        ) : null}
      </span>
    );
  }

  return (
    <li className={styles.mathlinguaListFileItem + ' ' + styles.sidePanelItem}>
      <Link
        to={`/${props.node.path}`}
        key={props.node.name}
        className={styles.link}
        onClick={() => {
          if (isOnMobile()) {
            dispatch(sidePanelVisibilityChanged(false));
          }
        }}
      >
        {props.node.name.replace('.math', '').replace('_', ' ')}
      </Link>
    </li>
  );
};