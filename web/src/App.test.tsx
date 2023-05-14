import React from 'react';

import { render, screen } from '@testing-library/react';
import { App } from './App';

describe('App', () => {
  it('displays text', () => {
    render(<App />);
    expect(screen.getByText(/some text/i)).toBeInTheDocument();
  });
});
