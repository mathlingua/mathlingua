import React from 'react';

import CloseIcon from '@rsuite/icons/Close';
import MenuIcon from '@rsuite/icons/Menu';

import { Group, TextArgumentData, TopLevelNodeKind } from '../../types';
import { GroupView } from './GroupView';
import { IdView } from './IdView';

import styles from './TopLevelNodeKindView.module.css';
import { TextBlockView } from './TextBlockView';
import { MarkdownView } from '../../design/MarkdownView';

export interface TopLevelNodeKindViewProps {
  node: TopLevelNodeKind;
  isOnSmallScreen: boolean;
  onSelectedSignature: (signature: string) => void;
  showCloseIcon: boolean;
  onCloseClicked: () => void;
}

export const TopLevelNodeKindView = (props: TopLevelNodeKindViewProps) => {
  const [showSource, setShowSource] = React.useState(false);

  const group = props.node as Group;
  const sections = group.Sections;
  if (sections) {
    let onlyHasCalledAndWritten = true;
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
                  if (sec.Name !== 'called' && sec.Name !== 'written') {
                    onlyHasCalledAndWritten = false;
                  }
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
      idText = `\\textrm{$${written}$}`;
    } else {
      isLatex = false;
      idText = (props.node as Group).Id;
      if (idText !== null && idText[0] === '\\') {
        idText = capitalize(idText.slice(1).replaceAll('.', ' '));
      }
    }

    let proofText = '';
    if (sections) {
      for (const sec of sections) {
        if (proofText.length > 0) {
          break;
        }

        if (sec.Name === 'Proof') {
          const args = sec.Args ?? [];
          for (const arg of args) {
            const innerArg = arg.Arg;
            if (innerArg && innerArg.Type === 'TextArgumentDataKind') {
              proofText = innerArg.Text;
              break;
            }
          }
        }
      }
    }

    return (
      <>
        <div className={styles.mathlinguaTopLevelEntity} style={{
            fontFamily: showSource ? '"Courier Prime", Courier' : undefined,
            fontSize: showSource ? '80%' : undefined,
        }}>
          <div className={styles.iconWrapper}>
            {props.showCloseIcon ? <CloseIcon className={styles.closeIcon} onClick={props.onCloseClicked} /> : <span></span>}
            <MenuIcon className={styles.switchIcon} onClick={() => setShowSource(src => !src)} />
          </div>
          <div className={styles.contentWrapper}
               style={{
                 overflowX: showSource ? 'auto' : undefined,
                 overflowY: showSource ? 'auto' : undefined,
               }}>
            <div className={styles.innerContentWrapper}>
              <IdView id={idText} isLatex={isLatex} idSrc={group.Id} showSource={showSource} />
              <GroupView
                node={props.node as any}
                showSource={showSource}
                indent={0}
                skipDocumentsSection={!showSource && onlyHasCalledAndWritten}
                onSelectedSignature={props.onSelectedSignature} />
            </div>
          </div>
        </div>
        {proofText.length > 0 &&
          <div className={styles.proofWrapper}>
            <span className={styles.proofTitle}>Proof</span>
            <MarkdownView text={`${proofText} $\\:\\:\\square$`}
                          className={styles.proofContent} />
          </div>
        }
      </>
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
