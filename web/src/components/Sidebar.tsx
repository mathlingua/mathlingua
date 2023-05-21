import React from 'react';

import styles from './Sidebar.module.css';

import { PathLabelPair } from '../types';
import { TreeNodeList } from './TreeNodeList';
import { buildTreeNode } from './tree';

export interface SidebarProps {
  selectedPath: string;
  allPaths: PathLabelPair[] | null;
  onSelect: (path: string) => void;
}

export function Sidebar(props: SidebarProps) {
  const { pathToNode, pathToParent } = React.useMemo(
    () => buildTreeNode(props.allPaths ?? []), [props.allPaths]);

    let selectedPath = props.selectedPath;
  while (selectedPath.endsWith('/')) {
    selectedPath = selectedPath.substring(0, selectedPath.length-1);
  }

  const selectedNode =
    selectedPath.endsWith('.math') ?
      pathToParent.get(selectedPath) :
      pathToNode.get(selectedPath);

  if (!selectedNode) {
    return <div className={styles.loading}>Loading...</div>;
  }

  return (
    <TreeNodeList
      node={selectedNode}
      selectedPath={selectedPath}
      onSelected={props.onSelect} />
  );
}
