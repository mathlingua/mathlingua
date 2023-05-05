import React, { useEffect } from 'react';

import { Tree } from 'rsuite';
import 'rsuite/dist/rsuite.css';

import { useFetch } from 'usehooks-ts';
import { useTheme } from '../hooks/theme';
import { PathsResponse } from '../types';
import { Theme } from '../base/theme';

export interface SidebarProps {
  selectedPath: string | undefined;
  onSelect: (path: string, isInit: boolean) => void;
}

export function Sidebar({ onSelect, selectedPath }: SidebarProps) {
  const theme = useTheme();
  const styles = getStyles(theme);

  const { data } = useFetch<PathsResponse>('/api/paths');
  const paths = data?.Paths
  const { tree: treeData, allPaths } = buildTreeNode(paths ?? []);

  const sortedPaths = allPaths.filter(path => path?.endsWith('.math'))
                              .map(path => path.split('/'))
                              .sort((a, b) => a.length - b.length)
                              .map(parts => parts.join('/'));
  const firstPath = sortedPaths[0];
  const [expandedValues, setExpandedValues] = React.useState([] as string[]);
  useEffect(() => {
    if (selectedPath === undefined && firstPath !== undefined) {
      onSelect(firstPath, true);
      setExpandedValues(decomposePath(firstPath));
    }
  }, [firstPath, onSelect, selectedPath]);

  if (data === undefined) {
    return <div style={styles.loading}>Loading...</div>;
  }

  return (
    <>
      <Tree
        style={styles.tree}
        data={treeData.children ?? []}
        onSelect={(val) => {
          const value = val.value as string;
          onSelect(value, false);
          const newExpandedValues = new Set(Array.from(expandedValues));
          if (expandedValues.indexOf(value) >= 0) {
            newExpandedValues.delete(value);
          } else {
            newExpandedValues.add(value);
          }
          setExpandedValues(Array.from(newExpandedValues));
        }}
        defaultValue={firstPath}
        expandItemValues={expandedValues}
        renderTreeNode={node => (
          <span style={node.children ? styles.bold : undefined}>
            {(node.label as string ?? '').replace(/_/g, ' ').replace('.math', '')}
          </span>
        )}
      />
    </>
  );
}

function getStyles(theme: Theme) {
  return {
    loading: {
      margin: theme.sizes.sizeXSmall,
    },
    tree: {
      maxHeight: 'fit-content',
      minHeight: `calc(100vh - ${theme.sizes.sizeXLarge}px)`,
      minWidth: theme.sizes.sidebarWidth,
      width: theme.sizes.sidebarWidth,
    },
    bold: {
      fontWeight: 'bold',
    },
  } as const;
}

interface TreeNode {
  label: string;
  value: string;
  children?: TreeNode[];
}

function buildTreeNode(paths: string[]): { tree: TreeNode; allPaths: string[]; } {
  const allPaths: string[] = [];
  const sentinal: TreeNode = {
    label: '',
    value: '',
    children: undefined,
  };
  for (const path of paths) {
    populateTreeNode(sentinal, path.split('/'), 0, allPaths);
  }
  return {
    tree: sentinal,
    allPaths,
  };
}

// Given "a/b/c/d" returns ["a", "a/b", "a/b/c", "a/b/c/d"]
function decomposePath(path: string | undefined): string[] {
  if (path === undefined) {
    return [];
  }

  const result: string[] = [];
  let buffer = '';
  for (const part of path.split('/')) {
    if (buffer.length > 0) {
      buffer += '/';
    }
    buffer += part;
    result.push(buffer);
  }
  return result;
}

function populateTreeNode(root: TreeNode, parts: string[], index: number, allPaths: string[]) {
  if (index >= parts.length) {
    return;
  }

  if (root.children === undefined) {
    root.children = [];
  }

  const cur = parts[index]!;
  const curIndex = root.children.findIndex((node) => {
    return node.label === cur;
  });

  let nextRoot: TreeNode;
  if (curIndex >= 0) {
    nextRoot = root.children[curIndex];
  } else {
    const value = parts.slice(0, index+1).join('/');
    nextRoot = {
      label: cur,
      value,
      children: undefined,
    };
    if (value.endsWith('.math')) {
      allPaths.push(value);
    }
    root.children.push(nextRoot);
  }

  populateTreeNode(nextRoot, parts, index+1, allPaths);
}
