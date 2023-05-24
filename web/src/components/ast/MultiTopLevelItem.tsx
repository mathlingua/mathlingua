import React from 'react';

import CloseIcon from '@rsuite/icons/Close';

import styles from './MultiTopLevelItem.module.css';

import { Group, TopLevelNodeKind } from '../../types';
import { TopLevelNodeKindView } from './TopLevelNodeKindView';

export interface MultiTopLevelItemProps {
  node: TopLevelNodeKind;
  isOnSmallScreen: boolean;
}

export const MultiTopLevelItem = (props: MultiTopLevelItemProps) => {
  const [selectedGroups, setSelectedGroups] = React.useState<Group[]>([]);

  const onSelectedSignature = async (signature: string) => {
    const res = await fetch(`/api/entry/signature/${encodeURIComponent(signature)}`);
    const json: { Error: string; Entry: Group; } = await res.json();
    if (json.Error !== null && json.Error !== undefined && json.Error === "") {
      if (!selectedGroups.find(grp => grp.Id === json.Entry.Id)) {
        setSelectedGroups(groups => [...groups, json.Entry]);
      }
    }
  };

  const onCloseClicked = (id: string|null) => {
    if (!id) {
      return;
    }
    setSelectedGroups(selectedGroups.filter(grp => grp.Id !== id));
  };

  return (
    <>
      <TopLevelNodeKindView
        node={props.node}
        isOnSmallScreen={props.isOnSmallScreen}
        onSelectedSignature={onSelectedSignature}
        showCloseIcon={false}
        onCloseClicked={() => {}} />
      {selectedGroups.length > 0 && 
        <div className={styles.selectedGroupsOuterWrapper}>
          <div className={styles.selectedGroupsInnerWrapper}>
            <span className={styles.closeIcon}
                  onClick={() => setSelectedGroups([])}>
              <CloseIcon />
            </span>
            {selectedGroups.map(group =>
              <TopLevelNodeKindView key={group.Id || Date.now()}
                                    node={group}
                                    isOnSmallScreen={props.isOnSmallScreen}
                                    onSelectedSignature={onSelectedSignature}
                                    showCloseIcon={true}
                                    onCloseClicked={() => onCloseClicked(group.Id)} />)}
          </div>
        </div>}
    </>
  );
}
