import styles from './App.module.css';

import React, { useState } from 'react';

import { HashRouter, Switch, Route } from 'react-router-dom';
import { ContentPanel } from './components/content-panel/ContentPanel';
import { ReferencePanel } from './components/reference/ReferencePanel';
import { TexTalkReferencePanel } from './components/reference/tex-talk-reference-panel/TexTalkReferencePanel';
import { ChalkTalkReferencePanel } from './components/reference/chalk-talk-reference-panel/ChalkTalkReferencePanel';

import CookieConsent from 'react-cookie-consent';

// import ReactGA from 'react-ga4';
let analyticsInitialized = false;
const showBanner = false;

export const App = () => {
  const [allowAnalytics, setAllowAnalytics] = useState(false);
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
                  onLocationChanged={(path) => {
                    if (allowAnalytics) {
                      if (!analyticsInitialized) {
                        // ReactGA.initialize('G-JPSYF6C3L8');
                        analyticsInitialized = true;
                      }
//                    ReactGA.event('page_view', {
//                      page_title: path
//                    });
                    }
                  }}
                />
              )}
            />
          </Switch>
        </div>
        {
          showBanner ?
          <CookieConsent
            buttonText='I agree'
            buttonStyle={{
              backgroundColor: '#70c273',
              color: '#ffffff',
              borderRadius: '3px'
            }}
            declineButtonText='I disagree'
            declineButtonStyle={{
              backgroundColor: '#d15858',
              color: '#ffffff',
              borderRadius: '3px'
            }}
            enableDeclineButton={true}
            acceptOnScroll={false}
            style={{
              fontFamily: 'Georgia, "Times New Roman", Times, serif',
              color: '#000000',
              borderTop: 'solid',
              borderTopColor: '#cccccc',
              borderTopWidth: '1px',
              background: '#f5f5f8'
            }}
            onAccept={() => setAllowAnalytics(true)}
            onDecline={() => setAllowAnalytics(false)}
            debug={true}>
            If you agree, this site will use cookies to analyze its traffic.
          </CookieConsent> : null
        }
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
