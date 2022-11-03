import React from 'react';

export interface IdViewProps {
  id: string | null;
}

export const IdView = (props: IdViewProps) => {
  if (props.id === null) {
    return null;
  }

  return (
    <div style={styles.mathlinguaId}>
      [{props.id}]
    </div>
  );
};

const styles = {
  mathlinguaId: {
    color: '#50a',
  }
};
