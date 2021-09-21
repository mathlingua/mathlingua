import styles from './App.module.css';

import React from 'react';
import { TopBar } from './components/topbar/TopBar';

import { HashRouter, Switch, Route } from 'react-router-dom';
import { ContentPanel } from './components/content-panel/ContentPanel';

export const App = () => {
  return (
    <HashRouter hashType="slash">
      <ErrorBoundary>
        <TopBar />
        <div className={styles.contentPanel}>
          <Switch>
            <Route path="/:relativePath(.*)" children={<ContentPanel />} />
          </Switch>
        </div>
      </ErrorBoundary>
    </HashRouter>
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
