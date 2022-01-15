import styles from './App.module.css';

import React, { useEffect, useState } from 'react';

import { HashRouter, Switch, Route } from 'react-router-dom';
import { ContentPanel } from './components/content-panel/ContentPanel';
import { ReferencePanel } from './components/reference/ReferencePanel';
import { TexTalkReferencePanel } from './components/reference/tex-talk-reference-panel/TexTalkReferencePanel';
import { ChalkTalkReferencePanel } from './components/reference/chalk-talk-reference-panel/ChalkTalkReferencePanel';

import CookieConsent from 'react-cookie-consent';
import * as api from './services/api';

import ReactGA from 'react-ga4';
let analyticsInitialized = false;

export const App = () => {
  const [allowAnalytics, setAllowAnalytics] = useState(false);
  const [googleAnalyticsId, setGoogleAnalyticsId] = useState(null as null | string);

  useEffect(() => {
    api.getConfiguration().then(config => {
      const id = config.googleAnalyticsId;
      if (id) {
        setGoogleAnalyticsId(id);
      }
    });
  }, []);

  const pageViewed = (path: string) => {
    if (allowAnalytics && googleAnalyticsId) {
      if (!analyticsInitialized) {
        ReactGA.initialize(googleAnalyticsId);
        analyticsInitialized = true;
      }
      ReactGA.event('page_view', {
        page_title: path
      });
    }
  };

  return (
    <HashRouter hashType="slash">
      <ErrorBoundary>
        <div className={styles.contentPanel}>
          <Switch>
            <Route exact path="/help">
              <ReferencePanel
                onLoad={() => pageViewed('/help')}/>
            </Route>
            <Route exact path="/help/expressionLanguage">
              <TexTalkReferencePanel
                onLoad={() => pageViewed('/help/expressionLanguage')}/>
            </Route>
            <Route exact path="/help/structuralLanguage">
              <ChalkTalkReferencePanel
                onLoad={() => pageViewed('/help/structuralLanguage')}/>
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
                  onLocationChanged={(path) => pageViewed(path)}
                />
              )}
            />
          </Switch>
        </div>
        {
          googleAnalyticsId ?
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
            debug={false}>
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
