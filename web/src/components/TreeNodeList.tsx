import React from 'react';

import UpIcon from '@rsuite/icons/ArrowUp';

import styles from './TreeNodeList.module.css';
import { TreeNode } from './tree';
import { Button } from '../design/Button';

export interface TreeNodeListProps {
  node: TreeNode;
  onSelect: (item: TreeNode) => void;
  onGoToParent: (parent: TreeNode) => void;
}

export function TreeNodeList(props: TreeNodeListProps) {
  const first = props.node.children?.find(node => node.children.length === 0);
  const [selectedItem, setSelectedItem] = React.useState(first);
  const onSelect = props.onSelect;
  React.useEffect(() => {
    if (first) {
      onSelect(first);
    }
  }, [first, onSelect]);
  return (
    <div className={styles.wrapper}>
      <div className={styles.outline}>
        {props.node.parent == null ? 'Outline' : null}
        {props.node.parent &&
          <Button ariaLabel='Go to parent directory'
                  onClick={() => {
                    props.onGoToParent(props.node.parent!);
                  }}
                  flat
                  className={styles.outlineButton}>
            {props.node.label}
            <UpIcon />
          </Button>}
      </div>
      {props.node.children.map(item => (
        <button className={getItemClassName(selectedItem?.path === item.path,
                                            item.children.length > 0)}
                key={item.path}
                onClick={(event) => {
                  event.preventDefault();
                  setSelectedItem(item);
                  props.onSelect(item);
                }}>
          {item.label}
        </button>
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
