import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { render, screen, wait } from '@testing-library/react';
import { Provider } from 'react-redux';
import { Home } from './Home';
import { store } from '../../store/store';

const server = setupServer(
  rest.get('/api/home', (req, res, ctx) => {
    return res(ctx.json({ homeHtml: 'Welcome to MathLingua' }));
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('loads home screen', async () => {
  render(
    <Provider store={store}>
      <Home />
    </Provider>
  );
  await wait(() => {
    const el = screen.getByTestId('home');
    expect(el).toBeInTheDocument();
    expect(el.innerHTML).toContain('Welcome to MathLingua');
    expect(el).toMatchSnapshot();
  });
});
