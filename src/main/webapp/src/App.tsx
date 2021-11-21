import styles from './App.module.css';

import React from 'react';

import { HashRouter, Switch, Route } from 'react-router-dom';
import { ContentPanel } from './components/content-panel/ContentPanel';
import { ReferencePanel } from './components/reference/ReferencePanel';
import { TexTalkReferencePanel } from './components/reference/tex-talk-reference-panel/TexTalkReferencePanel';
import { ChalkTalkReferencePanel } from './components/reference/chalk-talk-reference-panel/ChalkTalkReferencePanel';

export const App = () => {
  return (
    <HashRouter hashType="slash">
      <ErrorBoundary>
        <div className={styles.contentPanel}>
          <Switch>
            <Route exact path="/help">
              <ReferencePanel />
            </Route>
            <Route exact path="/help/expressionLanguage">
              <TexTalkReferencePanel />
            </Route>
            <Route exact path="/help/structuralLanguage">
              <ChalkTalkReferencePanel />
            </Route>
            <Route
              path="/:relativePath(.*)"
              render={(routerProps) => (
                <ContentPanel
                  redirect={(path) =>
                    routerProps.history.push({
                      pathname: path,
                    })
                  }
                />
              )}
            />
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
