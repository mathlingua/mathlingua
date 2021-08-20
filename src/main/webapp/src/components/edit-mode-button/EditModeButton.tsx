import { check } from '../../services/api';
import { errorResultsUpdated } from '../../store/errorResultsSlice';
import {
  isEditModeUpdated,
  selectIsEditMode,
} from '../../store/isEditModeSlice';
import { sidePanelVisibilityChanged } from '../../store/sidePanelVisibleSlice';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import styles from './EditModeButton.module.css';

export const EditModeButton = () => {
  const dispatch = useAppDispatch();
  const isEditMode = useAppSelector(selectIsEditMode);

  return (
    <button
      className={styles.editModeButton}
      onClick={async () => {
        const newEditMode = !isEditMode;
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
      {isEditMode ? 'View' : 'Edit'}
    </button>
  );
};
