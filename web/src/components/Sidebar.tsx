import React from 'react';

import styles from './Sidebar.module.css';

import { useFetch } from 'usehooks-ts';
import { PathsResponse } from '../types';
import { TreeNodeList } from './TreeNodeList';
import { TreeNode, buildTreeNode } from './tree';

export interface SidebarProps {
  onSelect: (path: TreeNode) => void;
}

export function Sidebar(props: SidebarProps) {
  const [selectedPathItem, setSelectedPathItem] = React.useState<TreeNode | null>(null);
  const { data } = useFetch<PathsResponse>('/api/paths');
  const paths = data?.Paths
  const { tree } = buildTreeNode(paths ?? []);

  if (data === undefined) {
    return <div className={styles.loading}>Loading...</div>;
  }

  return (
    <TreeNodeList
      node={selectedPathItem ?? tree}
      onSelect={(item) => {
        if (item.children.length > 0) {
          setSelectedPathItem(item);
        }
        props.onSelect(item);
      }}
      onGoToParent={(parent) => {
        setSelectedPathItem(parent);
      }} />
  );
}
