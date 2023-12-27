import React from 'react';

export interface DotProps {
  showSource: boolean;
}

export const Dot = (props: DotProps) => {
  if (props.showSource) {
    return <span>.</span>;
  }

  return (
    <span style={DOT_STYLE}>
      &#8729;
    </span>
  );
};

export const DOT_STYLE = {
  verticalAlign: 'center',
  width: '0.8ex',
  display: 'inline-block',
};
