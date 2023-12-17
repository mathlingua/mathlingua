import React from 'react';

export const Dot = () => {
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
