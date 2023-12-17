import React from 'react';
import { DOT_STYLE } from './Dot';

export const Space = () => {
  return (
    <span style={{...DOT_STYLE, opacity: 0}}>
      &#8729;
    </span>
  );
};
