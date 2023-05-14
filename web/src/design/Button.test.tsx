import React from 'react';

import { render, screen } from '@testing-library/react';
import { userEvent } from '@storybook/testing-library';

import { Button } from './Button';
import styles from './Button.module.css';

describe('Button', () => {
  it('should render correctly', () => {
    render(<Button>some text</Button>);
    expect(screen.queryByText(/some text/i)).toBeInTheDocument();
  });

  it('should handle onClick correctly', () => {
    const fn = jest.fn();
    render(<Button onClick={fn} />);
    const button = screen.getByRole('button');
    userEvent.click(button);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it('should specify aria-label if specified', () => {
    render(<Button ariaLabel='some-label'/>);
    expect(screen.getByLabelText('some-label')).toBeInTheDocument();
  });

  it('should use correct styling if flat=true', () => {
    render(<Button flat />);
    expect(screen.getByRole('button')).toHaveClass(styles.flat);
  });

  it('should use correct styling if flat=false', () => {
    render(<Button flat={false} />);
    expect(screen.getByRole('button')).toHaveClass(styles.regular);
  });
});
