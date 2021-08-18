import styles from './App.module.css';

import React from 'react';
import { Page } from './components/page/Page';
import { SidePanel } from './components/side-panel/SidePanel';
import { TopBar } from './components/topbar/TopBar';

import { BrowserRouter, StaticRouter, Switch, Route } from 'react-router-dom';
import { isStatic } from './services/api';

export const App = () => {
  if (isStatic()) {
    return (
      <StaticRouter>
        <ErrorBoundary>
          <TopBar />
          <SidePanel />
          <div className={styles.contentPanel}>
            <Switch>
              <Route path="/*" children={<Page />} />
            </Switch>
          </div>
        </ErrorBoundary>
      </StaticRouter>
    );
  }

  return (
    <BrowserRouter>
      <ErrorBoundary>
        <TopBar />
        <SidePanel />
        <div className={styles.contentPanel}>
          <Switch>
            <Route path="/*" children={<Page />} />
          </Switch>
        </div>
      </ErrorBoundary>
    </BrowserRouter>
  );
};

interface ErrorBoundaryState {
  hasError: boolean;
}

class ErrorBoundary extends React.Component<{}, ErrorBoundaryState> {
  constructor(props: {}) {
    super(props);
    this.state = {
      hasError: false,
    };
  }

  componentDidCatch(error: any, info: any) {
    this.setState({
      hasError: true,
    });
  }

  render() {
    if (this.state.hasError) {
      return (
        <p>
          An error occurred connecting to the MathLingua server. Are you running{' '}
          <code>mlg serve</code>?
        </p>
      );
    } else {
      return this.props.children;
    }
  }
}
