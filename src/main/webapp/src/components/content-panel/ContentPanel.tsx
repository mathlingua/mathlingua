import { Tab, Tabs, TabList, TabPanel } from 'react-tabs';
import 'react-tabs/style/react-tabs.css';

import styles from './ContentPanel.module.css';

import { useLocation } from 'react-router';
import { Page } from '../page/Page';
import { SidePanel } from '../side-panel/SidePanel';
import { selectIsEditMode } from '../../store/isEditModeSlice';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import {
  selectSidePanelVisible,
  sidePanelVisibilityChanged,
} from '../../store/sidePanelVisibleSlice';
// import { isOnMobile } from '../../support/util';
import React, { useCallback, useEffect, useRef, useState } from 'react';
import { isOnMobile } from '../../support/util';
import * as api from '../../services/api';
import { TopBar } from '../topbar/TopBar';
import { SignatureIndex } from '../signature-index/SignatureIndex';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes } from '@fortawesome/free-solid-svg-icons';
import { selectedTabPathUpdated, selectSelectedTabPath } from '../../store/selectedTabPathSlice';

export interface HashLocation {
  viewedPath: string;
  targetId: string;
}

function getHashLocation(location: {
  pathname: string;
  hash: string;
}): HashLocation {
  return {
    viewedPath: location.pathname.replace(/^\//, ''),
    targetId: location.hash.replace(/^#/, ''),
  };
}

export interface ContentPanelProps {
  redirect: (relativePath: string) => void;
  onLocationChanged: (path: string) => void;
}

export const ContentPanel = (props: ContentPanelProps) => {
  const hashLocation = getHashLocation(useLocation());
  const isEditMode = useAppSelector(selectIsEditMode);
  const isSidePanelVisible = useAppSelector(selectSidePanelVisible);

  useEffect(() => {
    props.onLocationChanged(hashLocation.viewedPath);
  }, [hashLocation]);

  useEffect(() => {
    if (hashLocation.viewedPath === '') {
      api.getFirstPath().then((path) => props.redirect(path));
    }
  }, []);

  // If the viewedPath is empty, then don't show anything.  The
  // effect above will redirect to the first page.
  if (hashLocation.viewedPath === '') {
    return null;
  }

  const innerPanel = isEditMode ? (
    <TwoColumnContent
      hashLocation={hashLocation}
      isSidePanelVisible={isSidePanelVisible}
    ></TwoColumnContent>
  ) : (
    <ThreeColumnContent
      hashLocation={hashLocation}
      startedWithSidePanelVisible={isSidePanelVisible}
    ></ThreeColumnContent>
  );

  return (
    <div>
      <TopBar />
      {innerPanel}
    </div>
  );
};

const ThreeColumnContent = (props: {
  hashLocation: HashLocation;
  startedWithSidePanelVisible: boolean;
}) => {
  const dispatch = useAppDispatch();
  const ref = useRef(null);

  const isZoomedInEnoughToHideSidebar = useCallback(
    () =>
      isOnMobile() ||
      (ref.current &&
        (ref.current as any).clientWidth >= 0.75 * window.screen.width),
    [ref]
  );

  const isSidePanelVisible = useAppSelector(selectSidePanelVisible);
  const [zoomedInEnoughToHideSidebar, setZoomedInEnoughToHideSidebar] =
    useState(isZoomedInEnoughToHideSidebar());

  useEffect(() => {
    const handleResize = () => {
      const newZoomedIn = isZoomedInEnoughToHideSidebar();
      if (props.startedWithSidePanelVisible) {
        if (newZoomedIn) {
          dispatch(sidePanelVisibilityChanged(false));
        } else {
          dispatch(sidePanelVisibilityChanged(true));
        }
      }
      setZoomedInEnoughToHideSidebar(newZoomedIn);
    };

    window.addEventListener('resize', handleResize);

    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const centerWidth = '45em';
  const sideWidth = `calc((100% - ${centerWidth}) / 2)`;
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'row',
        padding: 0,
        margin: 0,
      }}
    >
      <div
        style={{
          display:
            !zoomedInEnoughToHideSidebar || isSidePanelVisible
              ? 'block'
              : 'none',
          width: zoomedInEnoughToHideSidebar ? '100%' : sideWidth,
          padding: 0,
          marginLeft: 0,
          marginBottom: 0,
          marginTop: '0.25em',
        }}
      >
        {isSidePanelVisible ? (
          <div
            style={
              zoomedInEnoughToHideSidebar
                ? {}
                : { width: '100%', float: 'right' }
            }
          >
            <SidePanel viewedPath={props.hashLocation.viewedPath}
                       onOpenFileInTab={() => {}} />
          </div>
        ) : null}
      </div>
      <div
        ref={ref}
        style={{
          width: isOnMobile()
            ? isSidePanelVisible
              ? '0%'
              : centerWidth
            : centerWidth,
          maxWidth: '95%',
          marginTop: zoomedInEnoughToHideSidebar ? '0.5em' : '1em',
          marginLeft: 'auto',
          marginRight: 'auto',
        }}
      >
        {
          (props.hashLocation.viewedPath === 'index') ?
          <SignatureIndex /> :
          <Page
            viewedPath={props.hashLocation.viewedPath}
            targetId={props.hashLocation.targetId}
          />
        }
      </div>
      <div
        style={{
          width: zoomedInEnoughToHideSidebar ? '0%' : sideWidth,
          display: zoomedInEnoughToHideSidebar ? 'none' : 'block',
        }}
      ></div>
    </div>
  );
};

const TwoColumnContent = (props: {
  hashLocation: HashLocation;
  isSidePanelVisible: boolean;
}) => {
  const dispatch = useAppDispatch();
  const [locations, setLocations] = useState([] as HashLocation[]);

  const getViewedLocationIndex = (path: string) => {
    return locations.findIndex(location =>
      location.viewedPath === path);
  };

  const [selectedIndex, setSelectedIndex] = useState(
    Math.max(0, getViewedLocationIndex(props.hashLocation.viewedPath)));

  const selectIndex = (index: number) => {
    setSelectedIndex(index);
    const location = locations[index];
    if (location) {
      dispatch(selectedTabPathUpdated(location.viewedPath));
    }
  };

  useEffect(() => {
    const viewedLocation = props.hashLocation;
    const viewedLocationIndex = getViewedLocationIndex(viewedLocation.viewedPath);
    if (locations.length === 0) {
      setLocations([viewedLocation]);
    } else if (viewedLocationIndex >= 0) {
      // the selected path is already in the list so just switch to it
      selectIndex(viewedLocationIndex);
    } else {
      const newLocations: HashLocation[] = [];
      locations.filter(location =>
        location.viewedPath !== viewedLocation.viewedPath).forEach((location, index) => {
          if (index === selectedIndex) {
            newLocations.push(viewedLocation);
          } else {
            newLocations.push(location);
          }
        });
      setLocations(newLocations);
    }
  }, [props.hashLocation]);

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'row',
        padding: '0',
        margin: '0',
      }}
    >
      <div style={{ width: props.isSidePanelVisible ? '20%' : '0' }}>
        {props.isSidePanelVisible ? (
          <SidePanel viewedPath={props.hashLocation.viewedPath}
                     onOpenFileInTab={(path: string) => {
                       const index = getViewedLocationIndex(path);
                       if (index >= 0) {
                        selectIndex(index);
                       } else {
                         //  the path isn't in the list so add it
                         setLocations(locations.concat({
                           viewedPath: path,
                           targetId: ''
                         }));
                       }
                     }} />
        ) : null}
      </div>
      <div
        style={locations.length === 1 ? {
          width: props.isSidePanelVisible ? '80%' : '100%',
          marginTop: '0.5em',
          borderTop: 'solid',
          borderTopWidth: '1px',
          borderTopColor: '#aaaaaa',
        } : {
          width: props.isSidePanelVisible ? '80%' : '100%',
          marginTop: '0.5em',
          border: 'none',
        }}
      >
        <TabbedView locations={locations}
                    selectedIndex={selectedIndex}
                    setSelectedIndex={selectIndex}
                    onClose={(path) => {
                      setLocations(locations.filter(location =>
                        location.viewedPath !== path))
                    }} />
      </div>
    </div>
  );
};

const getLastPathLocation = (path: string) => {
  const parts = path.split('/');
  if (parts.length === 0) {
    return path;
  }

  return parts[parts.length - 1];
}

const TabbedView = (props: {
  selectedIndex: number | undefined;
  locations: HashLocation[];
  onClose: (path: string) => void;
  setSelectedIndex: (index: number) => void;
}) => {
  if (props.locations.length === 1) {
    const location = props.locations[0];
    return (
      <Page
        viewedPath={location.viewedPath}
        targetId={location.targetId}
      />
    );
  }

  return (
    <Tabs selectedIndex={props.selectedIndex}
          onSelect={(index) => props.setSelectedIndex(index)}>
      <TabList style={{ margin: '0' }}>
        {
          props.locations.map((location, index) =>
            <Tab key={index}
                 selectedClassName={styles.selectedTab}>
              {getLastPathLocation(location.viewedPath)}
              <button className={styles.button}>
                <FontAwesomeIcon
                  icon={faTimes}
                  style={{
                    filter: 'drop-shadow(0.45px 0.45px 0px rgba(0, 0, 0, 0.2))',
                  }}
                  onClick={() => props.onClose(location.viewedPath)}
                />
              </button>
            </Tab>)
        }
      </TabList>
      {
        props.locations.map((location, index) => <TabPanel key={index}>
          <Page
            viewedPath={location.viewedPath}
            targetId={location.targetId}
          />
        </TabPanel>)
      }
    </Tabs>
  );
};
