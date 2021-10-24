import { useLocation } from 'react-router';
import { Page } from '../page/Page';
import { SidePanel } from '../side-panel/SidePanel';
import { selectIsEditMode } from '../../store/isEditModeSlice';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import sidePanelVisibleSlice, {
  selectSidePanelVisible,
  sidePanelVisibilityChanged,
} from '../../store/sidePanelVisibleSlice';
// import { isOnMobile } from '../../support/util';
import { useCallback, useEffect, useRef, useState } from 'react';
import { isOnMobile } from '../../support/util';

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

  return isEditMode ? (
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
};

const ThreeColumnContent = (props: {
  hashLocation: HashLocation;
  startedWithSidePanelVisible: boolean;
}) => {
  const dispatch = useAppDispatch();
  const ref = useRef(null);

  const isZoomedInEnoughToHideSidebar = useCallback(
    () =>
      ref.current &&
      (ref.current as any).clientWidth >= 0.75 * window.screen.width,
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
          <div style={zoomedInEnoughToHideSidebar ? {} : { float: 'right' }}>
            <SidePanel viewedPath={props.hashLocation.viewedPath} />
          </div>
        ) : null}
      </div>
      <div
        ref={ref}
        style={{
          width: isOnMobile()
            ? isSidePanelVisible
              ? '0%'
              : '95%'
            : centerWidth,
          marginTop: zoomedInEnoughToHideSidebar ? '0.5em' : '1em',
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
