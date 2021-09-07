import { useState } from 'react';
import { EntityResult } from '../../services/api';
import { EntitiesPanel } from '../entities-panel/EntitiesPanel';
import { TopLevelEntry } from '../top-level-entry/TopLevelEntry';
import styles from './TopLevelEntityGroup.module.css';
import * as uuid from 'uuid';

export interface TopLevelEntityGroupProps {
  entity: EntityResult;
}

interface EntityResultWithId {
  id: string;
  entityResult: EntityResult;
}

export const TopLevelEntityGroup = (props: TopLevelEntityGroupProps) => {
  const [entitiesWithIds, setEntitiesWithIds] = useState(
    [] as EntityResultWithId[]
  );

  const onViewEntity = (entity: EntityResult) => {
    setEntitiesWithIds(
      entitiesWithIds.concat({
        id: uuid.v4(),
        entityResult: entity,
      })
    );
  };

  const onEntityClosed = (id: string) => {
    setEntitiesWithIds(
      entitiesWithIds.filter((container) => container.id !== id)
    );
  };

  return (
    <div className={styles.topLevelEntityGroup}>
      <TopLevelEntry
        id={props.entity.id}
        rawHtml={props.entity.rawHtml}
        renderedHtml={props.entity.renderedHtml}
        showCloseButton={false}
        onViewEntity={onViewEntity}
        onEntityClosed={onEntityClosed}
      />
      {entitiesWithIds.length > 0 ? (
        <div className={styles.triangleBorder}>
          <div className={styles.triangle}></div>
        </div>
      ) : null}
      <EntitiesPanel
        entityPanes={entitiesWithIds.map((container) => {
          return {
            id: container.id,
            topLevelEntry: (
              <TopLevelEntry
                id={container.id}
                showCloseButton={true}
                rawHtml={container.entityResult.rawHtml}
                renderedHtml={container.entityResult.renderedHtml}
                onViewEntity={onViewEntity}
                onEntityClosed={onEntityClosed}
              ></TopLevelEntry>
            ),
          };
        })}
        onCloseAll={() => setEntitiesWithIds([])}
      />
    </div>
  );
};
