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
} from '../../store/sidePanelVisibleSlice';
import { useEffect, useRef, useState } from 'react';
import * as api from '../../services/api';
import { TopBar } from '../topbar/TopBar';
import { SignatureIndex } from '../signature-index/SignatureIndex';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes } from '@fortawesome/free-solid-svg-icons';
import { selectedTabPathUpdated } from '../../store/selectedTabPathSlice';
import { isFullscreenUpdated, selectIsFullscreen } from '../../store/isFullscreenSlice';
import {
  faCaretUp,
  faCaretDown
} from '@fortawesome/free-solid-svg-icons';

import SplitPane from 'react-split-pane';
import { isOnMobile } from '../../support/util';
const Pane = require('react-split-pane/lib/Pane');

export interface HashLocation {
  viewedPath: string;
  targetId: string;
  line: number;
}

function getHashLocation(location: {
  pathname: string;
  hash: string;
  search: string;
}): HashLocation {
  return {
    viewedPath: location.pathname.replace(/^\//, ''),
    targetId: location.hash.replace(/^#/, ''),
    line: Number(location.search.replace(/^\?line=/, '') || 0)
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
  const isFullscreen = useAppSelector(selectIsFullscreen);
  const dispatch = useAppDispatch();

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
      {isFullscreen ? null : <TopBar />}
      {
        isEditMode ?
        <span style={{
            position: 'absolute',
            right: '0',
            paddingTop: '2ex',
            paddingRight: '1.5ex',
          }}>
          <button
            style={{
              border: 'none',
              background: 'transparent',
            }}
            onClick={() => {
              dispatch(isFullscreenUpdated(!isFullscreen))
          }}>
            <FontAwesomeIcon
              icon={isFullscreen ? faCaretDown : faCaretUp}
              style={{
                filter: 'drop-shadow(0.45px 0.45px 0px rgba(0, 0, 0, 0.2))',
                fontSize: '110%',
              }}
            />
          </button>
        </span> : null
      }
      {innerPanel}
    </div>
  );
};

const ThreeColumnContent = (props: {
  hashLocation: HashLocation;
  startedWithSidePanelVisible: boolean;
}) => {
  const isSidePanelVisible = useAppSelector(selectSidePanelVisible);
  const [isNarrow, setIsNarrow] = useState(isOnMobile());

  useEffect(() => {
    const updateIsNarrow = () => {
      setIsNarrow(isOnMobile());
    };
    window.addEventListener('resize', updateIsNarrow);
    return () => {
      window.removeEventListener('resize', updateIsNarrow);
    };
  }, []);

  if (isNarrow && window.innerWidth < 1024) {
    return <div style={{
      marginTop: '0.75em',
      display: 'grid',
      gridTemplateColumns: '2.5% 95% 2.5%',
    }}>
      <div>
        {
          isSidePanelVisible ?
          <div style={{
              position: 'relative',
              zIndex: '10',
              background: '#ffffff',
              borderColor: '#dddddd',
              boxShadow: '0px 1px 5px rgba(0, 0, 0, .2)',
              height: '100%',
              width: 'max-content',
              minWidth: 'fit-content',
              paddingTop: '0.3em',
            }}>
            <SidePanel viewedPath={props.hashLocation.viewedPath}
                       onOpenFileInTab={() => {}} />
          </div> : null
        }
      </div>
      <div>
        {
          (props.hashLocation.viewedPath === 'index') ?
          <SignatureIndex /> :
          <Page
            viewedPath={props.hashLocation.viewedPath}
            viewedLine={props.hashLocation.line}
            targetId={props.hashLocation.targetId}
            onOpenFileInTab={() => {}}
          />
        }
      </div>
      <div></div>
    </div>;
  }

  return <div style={{
    marginTop: '0.75em',
    display: 'grid',
    gridTemplateColumns: '22.5% 55% 22.5%',
  }}>
    <div style={{ marginTop: '-0.3em' }}>
      { isSidePanelVisible ?
        <SidePanel viewedPath={props.hashLocation.viewedPath}
                   onOpenFileInTab={() => {}} /> : null }
    </div>
    <div>
      {
        (props.hashLocation.viewedPath === 'index') ?
        <SignatureIndex /> :
        <Page
          viewedPath={props.hashLocation.viewedPath}
          viewedLine={props.hashLocation.line}
          targetId={props.hashLocation.targetId}
          onOpenFileInTab={() => {}}
        />
      }
    </div>
    <div></div>
  </div>;
};

const TwoColumnContent = (props: {
  hashLocation: HashLocation;
  isSidePanelVisible: boolean;
}) => {
  const dispatch = useAppDispatch();
  const [locations, setLocations] = useState([] as HashLocation[]);
  const [sidePanelSize, setSidePanelSize] = useState('20%');

  const getViewedLocationIndex = (path: string) => {
    return locations.findIndex(location =>
      location.viewedPath === path);
  };

  const [selectedIndex, setSelectedIndex] = useState(
    Math.max(0, getViewedLocationIndex(props.hashLocation.viewedPath)));

  // the react-split-pane resizer for the side panel is 0.5em too
  // high and the only way, it appears, to fix that is to find the
  // resizer and change its style dynamically
  useEffect(() => {
    const divs = document.getElementsByTagName('div');
    for (let i=0; i<divs.length; i++) {
      const div = divs[i];
      if (div?.dataset?.type === 'Resizer') {
        div.style.marginTop = '0.5em';
      }
    }
  }, [props.isSidePanelVisible]);

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
      const newSelectedIndex = selectedIndex >= locations.length ? 0 : selectedIndex;
      // make a copy of the locations and update `selectedIndex` to point to `viewedLocation`
      const newLocations: HashLocation[] = locations.map(location => ({
        viewedPath: location.viewedPath,
        targetId: location.targetId,
        line: location.line
      }));
      newLocations[newSelectedIndex] = {
        viewedPath: viewedLocation.viewedPath,
        targetId: viewedLocation.targetId,
        line: viewedLocation.line
      };
      setLocations(newLocations);
      setSelectedIndex(newSelectedIndex);
    }
  }, [props.hashLocation]);

  const openFileInTab = (path: string, line?: number) => {
    const index = getViewedLocationIndex(path);
    if (index >= 0) {
      if (line !== undefined) {
        setLocations(locations.map((location, i) => {
          if (i == index) {
            return {
              viewedPath: location.viewedPath,
              targetId: location.targetId,
              line
            };
          } else {
            return location;
          }
        }));
      }
      selectIndex(index);
    } else {
      //  the path isn't in the list so add it
      const newLocations = locations.concat({
        viewedPath: path,
        targetId: '',
        line: line ?? 0
      });
      setLocations(newLocations);

      // If `line !== undefined` then the opening of a new tab was because
      // the user used the context menu to look for the definition of a
      // signature.  In that case, a new tab was opened in the background as
      // the last tab, so select that tab.
      if (line !== undefined) {
        setSelectedIndex(newLocations.length - 1);
      }
    }
  };

  return <SplitPane onChange={(size) => setSidePanelSize(`${size}px`)}>
    {props.isSidePanelVisible ? (
      <Pane initialSize={sidePanelSize}>
        <SidePanel viewedPath={props.hashLocation.viewedPath}
                   onOpenFileInTab={openFileInTab} />
      </Pane>
    ) : null}
    <Pane>
      <div style={{
          marginTop: '0.5em',
          borderTop: 'solid',
          borderTopWidth: '1px',
          borderTopColor: '#aaaaaa',
        }}>
        <TabbedView locations={locations}
                    selectedIndex={selectedIndex}
                    setSelectedIndex={selectIndex}
                    onClose={(path) => {
                      setLocations(locations.filter(location =>
                        location.viewedPath !== path))
                    }}
                    onOpenFileInTab={openFileInTab} />
      </div>
    </Pane>
  </SplitPane>;
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
  onOpenFileInTab: (path: string, line?: number) => void;
}) => {
  if (props.locations.length === 1) {
    const location = props.locations[0];
    return (
      <Page
        viewedPath={location.viewedPath}
        viewedLine={location.line}
        targetId={location.targetId}
        onOpenFileInTab={props.onOpenFileInTab}
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
            viewedLine={location.line}
            targetId={location.targetId}
            onOpenFileInTab={props.onOpenFileInTab}
          />
        </TabPanel>)
      }
    </Tabs>
  );
};
