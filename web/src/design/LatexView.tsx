import React from 'react';

declare const katex: any;

export interface LatexViewProps {
  latex: string;
  color: string;
}

export const LatexView = (props: LatexViewProps) => {
  return (
    <span style={{ color: props.color, }}
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
