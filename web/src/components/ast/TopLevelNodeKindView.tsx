import React from 'react';

import { Group, TextArgumentData, TopLevelNodeKind } from '../../types';
import { GroupView } from './GroupView';
import { TextBlockView } from './TextBlockView';
import { IdView } from './IdView';

import styles from './TopLevelNodeKindView.module.css';

export interface TopLevelNodeKindViewProps {
  node: TopLevelNodeKind;
  isOnSmallScreen: boolean;
}

export const TopLevelNodeKindView = (props: TopLevelNodeKindViewProps) => {
  const sections = (props.node as Group).Sections;
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
      idText = `\\textrm{$${capitalize(written)}$}`;
    } else {
      isLatex = false;
      idText = (props.node as Group).Id;
      if (idText !== null && idText[0] === '\\') {
        idText = capitalize(idText.slice(1).replaceAll('.', ' '));
      }
    }
    return (
      <div className={styles.mathlinguaTopLevelEntity}>
        <IdView id={idText} isLatex={isLatex} />
        <GroupView node={props.node as any} indent={0} />
      </div>
    );
  } else {
    return (
      <div className={styles.mathlinguaTopLevelTextBlock}>
        <TextBlockView node={props.node as any} />
      </div>
    );
  }
};

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
