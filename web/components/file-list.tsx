import { GroupCard } from "./group-card";
import styles from "./file-list.module.css";
import {
  buildFileBrowserEntries,
  formatFileLabel,
  formatPathSegment,
  makeFileAnchor,
  makeGroupAnchor,
  parentDirectory,
} from "../lib/presenter";
import { FileView } from "../lib/types";

interface FileListProps {
  currentDirectory: string;
  files: FileView[];
  isOutlineOpen: boolean;
  onNavigateDirectory: (directory: string) => void;
  onSelectFile: (fileIndex: number) => void;
  selectedFileIndex: number;
}

export function FileList({
  currentDirectory,
  files,
  isOutlineOpen,
  onNavigateDirectory,
  onSelectFile,
  selectedFileIndex,
}: FileListProps) {
  if (files.length === 0) {
    return (
      <section className={styles.emptyState}>
        <h2>No renderable files found</h2>
        <p>Add Mathlingua files under your collection content directory.</p>
      </section>
    );
  }

  const selectedFile = files[selectedFileIndex] ?? files[0];
  const entries = buildFileBrowserEntries(files, currentDirectory);

  return (
    <div
      className={
        isOutlineOpen
          ? `${styles.readerLayout} ${styles.outlineOpen}`
          : `${styles.readerLayout} ${styles.outlineClosed}`
      }
    >
      <aside className={styles.outlinePanel}>
        {currentDirectory ? (
          <button
            className={styles.outlineBack}
            onClick={() =>
              onNavigateDirectory(parentDirectory(currentDirectory))
            }
            type="button"
          >
            <span aria-hidden="true" className={styles.outlineBackChevron} />
            {formatPathSegment(currentDirectory.split("/").at(-1) ?? "")}
          </button>
        ) : null}
        <nav>
          <ul className={styles.outlineList}>
            {entries.map((entry) => (
              <li key={`${entry.kind}-${entry.path}`}>
                {entry.kind === "directory" ? (
                  <button
                    className={`${styles.outlineLink} ${styles.outlineLinkDirectory}`}
                    onClick={() => onNavigateDirectory(entry.path)}
                    type="button"
                  >
                    <span>{entry.label}</span>
                    <span
                      aria-hidden="true"
                      className={styles.outlineLinkChevron}
                    />
                  </button>
                ) : (
                  <button
                    className={
                      entry.fileIndex === selectedFileIndex
                        ? `${styles.outlineLink} ${styles.outlineLinkActive}`
                        : styles.outlineLink
                    }
                    onClick={() => onSelectFile(entry.fileIndex)}
                    type="button"
                  >
                    {formatFileLabel(entry.path)}
                  </button>
                )}
              </li>
            ))}
          </ul>
        </nav>
      </aside>
      <section className={styles.documentStream}>
        <article
          className={styles.fileSection}
          id={makeFileAnchor(selectedFile.path)}
          key={selectedFile.path}
        >
          <div className={styles.groupStream}>
            {selectedFile.items.map((item, itemIndex) => (
              <GroupCard
                anchorId={makeGroupAnchor(selectedFileIndex, itemIndex)}
                group={item}
                key={`${selectedFile.path}-${item.kind}-${itemIndex}`}
              />
            ))}
          </div>
        </article>
      </section>
    </div>
  );
}
