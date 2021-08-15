import { useState } from 'react';
import { EntityResult } from '../../services/api';
import { EntitiesPanel } from '../entities-panel/EntitiesPanel';
import { TopLevelEntry } from '../top-level-entry/TopLevelEntry';
import styles from './TopLevelEntityGroup.module.css';

export interface TopLevelEntityGroupProps {
  entity: EntityResult;
}

export const TopLevelEntityGroup = (props: TopLevelEntityGroupProps) => {
  const [entities, setEntities] = useState([] as EntityResult[]);

  const onViewEntity = (entity: EntityResult) => {
    setEntities(entities.concat(entity));
  };

  const onEntityClosed = (id: string) => {
    setEntities(entities.filter((entity) => entity.id !== id));
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
      {entities.length > 0 ? (
        <div className={styles.triangleBorder}>
          <div className={styles.triangle}></div>
        </div>
      ) : null}
      <EntitiesPanel
        onViewEntity={onViewEntity}
        onEntityClosed={onEntityClosed}
        entities={entities}
        onCloseAll={() => setEntities([])}
      />
    </div>
  );
};
