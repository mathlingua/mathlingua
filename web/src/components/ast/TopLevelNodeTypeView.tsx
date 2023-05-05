import React from 'react';
import { useTheme } from '../../hooks/theme';
import { TopLevelNodeType } from '../../types';
import { GroupView } from './GroupView';
import { TextBlockView } from './TextBlockView';
import { Theme } from '../../base/theme';

export interface TopLevelNodeTypeViewProps {
  node: TopLevelNodeType;
  isOnSmallScreen: boolean;
}

export const TopLevelNodeTypeView = (props: TopLevelNodeTypeViewProps) => {
  const theme = useTheme();
  const isGroup = (props.node as any).Sections;
  const styles = getTopLevelNodeTypeViewStyles(theme);
  if (isGroup) {
    return (
      <div style={styles.mathlinguaTopLevelEntity}>
        <GroupView node={props.node as any} indent={0} />
      </div>
    );
  } else {
    return (
      <div style={styles.mathlinguaTopLevelTextBlock}>
        <TextBlockView node={props.node as any} />
      </div>
    );
  }
};

function getTopLevelNodeTypeViewStyles(theme: Theme) {
  return {
    mathlinguaTopLevelEntity: {
      fontFamily: 'monospace',
      padding: '2ex',
      minWidth: '50%',
      width: 'fit-content',
      height: 'max-content',
      overflow: 'auto',
      marginTop: '2ex',
      marginBottom: '2ex',
      marginLeft: 'auto',
      marginRight: 'auto',
      border: 'solid',
      borderColor: theme.colors.border,
      borderWidth: 1,
      borderRadius: 2,
      boxShadow: `${theme.colors.innerShadow} 0px 2px 5px 0px, ${theme.colors.outerShadow} 0px 1px 1px 0px`,
    },
    mathlinguaTopLevelTextBlock: {
      color: '#555555',
      maxWidth: '100%',
    },
  };
}
