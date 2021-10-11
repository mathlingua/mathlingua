import { check } from '../../services/api';
import { errorResultsUpdated } from '../../store/errorResultsSlice';
import {
  isEditModeUpdated,
  selectIsEditMode,
} from '../../store/isEditModeSlice';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import styles from './EditModeButton.module.css';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faEdit, faEye } from '@fortawesome/free-solid-svg-icons';

export const EditModeButton = () => {
  const dispatch = useAppDispatch();
  const isEditMode = useAppSelector(selectIsEditMode);

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        padding: 0,
        marginLeft: 0,
        marginRight: 0,
        marginTop: 0,
        marginBottom: 'auto',
      }}
    >
      <button
        className={styles.editModeButton}
        onClick={async () => {
          dispatch(isEditModeUpdated(!isEditMode));
          const res = await check();
          dispatch(
            errorResultsUpdated(
              res.errors.map((err) => ({
                row: err.row,
                column: err.column,
                message: err.message,
                relativePath: err.path,
              }))
            )
          );
        }}
      >
        {isEditMode ? (
          <FontAwesomeIcon icon={faEye} />
        ) : (
          <FontAwesomeIcon icon={faEdit} />
        )}
      </button>
    </div>
  );
};
