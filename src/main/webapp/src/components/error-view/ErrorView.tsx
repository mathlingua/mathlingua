import styles from './ErrorView.module.css';

interface ErrorViewProps {
  message: string;
}

export function ErrorView(props: ErrorViewProps) {
  return (
    <div className={styles.errorMessage}>
      <span className={styles.errorLabel}>ERROR:</span> {props.message}
    </div>
  );
}
