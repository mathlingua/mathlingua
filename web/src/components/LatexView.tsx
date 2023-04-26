import React from 'react';

declare const katex: any;

export interface LatexViewProps {
  latex: string;
  color?: string;
}

export const LatexView = (props: LatexViewProps) => {
  const styles = getStyles(props.color ?? 'black');
  return (
    <span style={styles.mathlinguaFormulation}
          dangerouslySetInnerHTML={{
            __html: katex.renderToString(removeQuestionMarks(props.latex), {
              throwOnError: false,
            }),
          }}>
    </span>
  );
};

function removeQuestionMarks(text: string): string {
  return text.replace(/([a-zA-Z0-9]*)(-|=|\+)?\?/g, '$1');
}

function getStyles(color: string) {
  return {
    mathlinguaFormulation: {
      color,
    }
  };
}
