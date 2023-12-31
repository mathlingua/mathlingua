import React from 'react';

import UpIcon from '@rsuite/icons/ArrowUp';

import styles from './TreeNodeList.module.css';
import { TreeNode } from './tree';
import { Link, useNavigate } from 'react-router-dom';

export interface TreeNodeListProps {
  selectedPath: string;
  node: TreeNode;
  onSelected(path: string): void;
}

export function TreeNodeList(props: TreeNodeListProps) {
  const navigate = useNavigate();

  React.useEffect(() => {
    if (!props.selectedPath.endsWith('.math')) {
      const first = props.node.children.find(n => n.path.endsWith('.math'));
      if (first) {
        navigate(first.path);
      }
    }
  }, [props.selectedPath]);

  return (
    <div className={styles.wrapper}>
      <div className={styles.outline}>
        {props.node.parent == null ? 'Outline' : null}
        {props.node.parent &&
          <Link to={props.node.parent.path}
                className={styles.outlineButton}
                onClick={() => props.onSelected(props.node.parent!.path)}>
            {props.node.label}
            <UpIcon className={styles.upIcon} />
          </Link>}
      </div>
      {props.node.children.map(item => (
        <Link to={item.path}
              className={getItemClassName(props.selectedPath === item.path,
                                          item.children.length > 0)}
              key={item.path}
              onClick={() => props.onSelected(item.path)}>
          {item.label}
        </Link>
      ))}
    </div>
  );
}

function getItemClassName(selected: boolean, isDirectory: boolean): string {
  let classes = selected ? `${styles.item} ${styles.selected}` : styles.item;
  if (isDirectory) {
    classes += ' ' + styles.bold;
  }
  return classes;
}
