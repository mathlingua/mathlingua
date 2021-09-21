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
    <button
      className={styles.editModeButton}
      onClick={async () => {
        const newEditMode = !isEditMode;
        if (newEditMode) {
          document.body.style.position = 'fixed';
        } else {
          document.body.style.position = 'unset';
        }
        dispatch(isEditModeUpdated(newEditMode));
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
  );
};
