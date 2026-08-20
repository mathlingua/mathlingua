import type { CSSProperties } from "react";
import type { TypeEntryView } from "../lib/types";
import styles from "./type-annotations.module.css";

interface TypeAnnotationsProps {
  entries: TypeEntryView[];
}

/** Shows the checker result for one formulation and its nested expressions. */
export function TypeAnnotations({ entries }: TypeAnnotationsProps) {
  if (entries.length === 0) {
    return null;
  }

  return (
    <div aria-label="Resolved types" className={styles.panel} role="note">
      {entries.map((entry, index) => (
        <div
          className={styles.row}
          key={`${entry.depth}-${entry.text}-${index}`}
          style={
            {
              "--type-depth": entry.depth,
            } as CSSProperties
          }
        >
          <code className={styles.expression}>{entry.text}</code>
          <span aria-hidden="true" className={styles.separator}>
            :
          </span>
          <span
            className={
              entry.types.length > 0 ? styles.types : styles.typesUnknown
            }
          >
            {entry.types.length > 0
              ? entry.types.join(", ")
              : "no type resolved"}
          </span>
        </div>
      ))}
    </div>
  );
}
