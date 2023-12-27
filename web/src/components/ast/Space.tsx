import React from 'react';
import { DOT_STYLE } from './Dot';

export interface SpaceProps {
  showSource: boolean;
}

export const Space = (props: SpaceProps) => {
  if (props.showSource) {
    return <span>&nbsp;</span>;
  }

  return (
    <span style={{...DOT_STYLE, opacity: 0}}>
      &#8729;
    </span>
  );
};
