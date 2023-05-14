import React from 'react';

import styles from './Button.module.css';

export interface ButtonProps {
  flat?: boolean;
  onClick?: () => void;
  ariaLabel?: string;
  children?: React.ReactNode;
  className?: string;
}

export const Button = (props: ButtonProps = {
  flat: true,
  onClick: () => {},
}) => {
  const className = props.flat ? styles.flat : styles.regular;
  return (
    <button aria-label={props.ariaLabel}
            onClick={props.onClick}
            className={props.className !== undefined ?
              `${className} ${props.className}` : className}>
      {props.children}
    </button>
    );
};
