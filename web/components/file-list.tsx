import { GroupCard } from "./group-card";
import {
  buildFileBrowserEntries,
  formatFileLabel,
  formatPathSegment,
  makeFileAnchor,
  makeGroupAnchor,
  parentDirectory,
} from "../lib/presenter";
import { FileView } from "../lib/types";

type FileListProps = {
  currentDirectory: string;
  files: FileView[];
  isOutlineOpen: boolean;
  onNavigateDirectory: (directory: string) => void;
  onSelectFile: (fileIndex: number) => void;
  selectedFileIndex: number;
};

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
      <section className="empty-state">
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
          ? "reader-layout reader-layout--outline-open"
          : "reader-layout reader-layout--outline-closed"
      }
    >
      <aside className="outline-panel">
        {currentDirectory ? (
          <button
            className="outline-back"
            onClick={() => onNavigateDirectory(parentDirectory(currentDirectory))}
            type="button"
          >
            <span aria-hidden="true" className="outline-back__chevron" />
            {formatPathSegment(currentDirectory.split("/").at(-1) ?? "")}
          </button>
        ) : null}
        <nav>
          <ul className="outline-list">
            {entries.map((entry) => (
              <li key={`${entry.kind}-${entry.path}`}>
                {entry.kind === "directory" ? (
                  <button
                    className="outline-link outline-link--directory"
                    onClick={() => onNavigateDirectory(entry.path)}
                    type="button"
                  >
                    <span>{entry.label}</span>
                    <span
                      aria-hidden="true"
                      className="outline-link__chevron"
                    />
                  </button>
                ) : (
                  <button
                    className={
                      entry.fileIndex === selectedFileIndex
                        ? "outline-link outline-link--active"
                        : "outline-link"
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
      <section className="document-stream">
        <article
          className="file-section"
          id={makeFileAnchor(selectedFileIndex)}
          key={selectedFile.path}
        >
          <div className="group-stream">
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
