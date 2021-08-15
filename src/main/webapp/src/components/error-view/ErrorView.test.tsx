import { render, screen } from '@testing-library/react';
import { ErrorView } from './ErrorView';

test('it renders', () => {
  render(<ErrorView message="some custom error" />);
  const el = screen.getByText('some custom error');
  expect(el).toBeInTheDocument();
  expect(el).toMatchSnapshot();
});
