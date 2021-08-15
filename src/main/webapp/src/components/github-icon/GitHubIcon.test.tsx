import { render } from '@testing-library/react';
import { GitHubIcon } from './GitHubIcon';

test('is rendered', () => {
  const el = render(<GitHubIcon />);
  expect(el.getByTitle('GitHub')).toBeInTheDocument();
  expect(el).toMatchSnapshot();
});
