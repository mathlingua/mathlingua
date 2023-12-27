import React from 'react';
import { Space } from './Space';

export interface CommaProps {
  showSource: boolean;
}

export const Comma = (props: CommaProps) => {
  return (
    <>
      ,<Space showSource={props.showSource} />
    </>
  );
};
