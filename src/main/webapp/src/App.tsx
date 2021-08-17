import styles from './App.module.css';

import React from 'react';
import { Page } from './components/page/Page';
import { SidePanel } from './components/side-panel/SidePanel';
import { TopBar } from './components/topbar/TopBar';

import { BrowserRouter as Router, Switch, Route } from 'react-router-dom';

export const App = () => {
  return (
    <Router>
      <ErrorBoundary>
        <TopBar />
        <SidePanel />
        <div className={styles.contentPanel}>
          <Switch>
            <Route path="/*" children={<Page />} />
          </Switch>
        </div>
      </ErrorBoundary>
    </Router>
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
