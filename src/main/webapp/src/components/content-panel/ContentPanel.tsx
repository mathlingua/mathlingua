import { useLocation } from 'react-router';
import { Page } from '../page/Page';
import { SidePanel } from '../side-panel/SidePanel';
import { selectIsEditMode } from '../../store/isEditModeSlice';
import { useAppSelector } from '../../support/hooks';
import { selectSidePanelVisible } from '../../store/sidePanelVisibleSlice';
import { isOnMobile, isOnWideScreen } from '../../support/util';

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

export const ContentPanel = () => {
  const hashLocation = getHashLocation(useLocation());
  const isEditMode = useAppSelector(selectIsEditMode);
  const isSidePanelVisible = useAppSelector(selectSidePanelVisible);
  const isMobile = isOnMobile();

  return isEditMode ? (
    <TwoColumnContent
      hashLocation={hashLocation}
      isSidePanelVisible={isSidePanelVisible}
    ></TwoColumnContent>
  ) : (
    <ThreeColumnContent
      isMobile={isMobile}
      hashLocation={hashLocation}
      isSidePanelVisible={isSidePanelVisible}
    ></ThreeColumnContent>
  );
};

const ThreeColumnContent = (props: {
  isMobile: boolean;
  hashLocation: HashLocation;
  isSidePanelVisible: boolean;
}) => {
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
            !props.isMobile || props.isSidePanelVisible ? 'block' : 'none',
          width: props.isMobile ? '100%' : isOnWideScreen() ? '30%' : '25%',
          padding: 0,
          marginLeft: 0,
          marginBottom: 0,
          marginTop: '0.25em',
        }}
      >
        {props.isSidePanelVisible ? (
          <div style={isOnMobile() ? {} : { float: 'right' }}>
            <SidePanel viewedPath={props.hashLocation.viewedPath} />
          </div>
        ) : null}
      </div>
      <div
        style={{
          width: props.isMobile
            ? props.isSidePanelVisible
              ? '0%'
              : '95%'
            : isOnWideScreen()
            ? '40%'
            : '50%',
          marginTop: props.isMobile ? '0.5em' : '1em',
          marginLeft: 'auto',
          marginRight: 'auto',
        }}
      >
        <Page
          viewedPath={props.hashLocation.viewedPath}
          targetId={props.hashLocation.targetId}
        />
      </div>
      <div
        style={{
          width: props.isMobile ? '0%' : isOnWideScreen() ? '30%' : '25%',
          display: props.isMobile ? 'none' : 'block',
        }}
      ></div>
    </div>
  );
};

const TwoColumnContent = (props: {
  hashLocation: HashLocation;
  isSidePanelVisible: boolean;
}) => {
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
          <SidePanel viewedPath={props.hashLocation.viewedPath} />
        ) : null}
      </div>
      <div
        style={{
          width: props.isSidePanelVisible ? '80%' : '100%',
          border: 'solid',
          borderColor: '#cccccc',
          borderWidth: '1px',
          marginTop: '0.5em',
        }}
      >
        <Page
          viewedPath={props.hashLocation.viewedPath}
          targetId={props.hashLocation.targetId}
        />
      </div>
    </div>
  );
};
