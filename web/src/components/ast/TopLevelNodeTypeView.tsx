import React from 'react';
import { useTheme } from '../../hooks/theme';
import { Group, TextArgumentData, TopLevelNodeType } from '../../types';
import { GroupView } from './GroupView';
import { TextBlockView } from './TextBlockView';
import { Theme } from '../../base/theme';
import { IdView } from './IdView';

export interface TopLevelNodeTypeViewProps {
  node: TopLevelNodeType;
  isOnSmallScreen: boolean;
}

export const TopLevelNodeTypeView = (props: TopLevelNodeTypeViewProps) => {
  const theme = useTheme();
  const sections = (props.node as Group).Sections;
  const styles = getTopLevelNodeTypeViewStyles(theme);
  if (sections) {
    let called: string|undefined = undefined;
    let written: string|undefined = undefined;
    if (sections !== null){
      for (const sec of sections) {
        if (sec.Name === 'Documented') {
          if (sec.Args !== null) {
            for (const arg of sec.Args) {
              const argSections = (arg.Arg as Group).Sections
              if (argSections !== null) {
                for (const sec of argSections) {
                  if (called === undefined &&
                      sec.Name === 'called' &&
                      sec.Args !== null &&
                      sec.Args.length > 0) {
                    called = (sec.Args[0].Arg as TextArgumentData).Text;
                  } else if (written === undefined &&
                      sec.Name === 'written' &&
                      sec.Args !== null &&
                      sec.Args.length > 0) {
                    written = (sec.Args[0].Arg as TextArgumentData).Text;
                  }
                }
              }
            }
          }
          break;
        }
      }
    }
    let isLatex = false;
    let idText: string | null = null;
    if (called) {
      isLatex = true;
      idText = `\\textrm{${capitalize(called)}}`;
    } else if (written) {
      isLatex = true;
      idText = `\\textrm{${capitalize(written)}}`;
    } else {
      isLatex = false;
      idText = (props.node as Group).Id;
      if (idText !== null && idText[0] === '\\') {
        idText = capitalize(idText.slice(1).replaceAll('.', ' '));
      }
    }
    return (
      <div style={styles.mathlinguaTopLevelEntity}>
        <IdView id={idText} isLatex={isLatex} />
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

function capitalize(text: string): string {
  if (text.length === 0) {
    return text;
  }

  const first = text.charAt(0);
  const firstCapitalized = first.toUpperCase();
  if (firstCapitalized === first) {
    // the text is already capitalized
    return text;
  }

  const tail = text.slice(1);
  return `${firstCapitalized}${tail}`;
}
