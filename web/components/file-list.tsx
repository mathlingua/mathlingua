import { GroupCard } from "./group-card";
import {
  formatFileLabel,
  makeFileAnchor,
  makeGroupAnchor,
} from "../lib/presenter";
import { FileView } from "../lib/types";

type FileListProps = {
  files: FileView[];
  isOutlineOpen: boolean;
  onSelectFile: (fileIndex: number) => void;
  selectedFileIndex: number;
};

export function FileList({
  files,
  isOutlineOpen,
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

  return (
    <div
      className={
        isOutlineOpen
          ? "reader-layout reader-layout--outline-open"
          : "reader-layout reader-layout--outline-closed"
      }
    >
      <aside className="outline-panel">
        <p className="outline-title">Outline</p>
        <nav>
          <ul className="outline-list">
            {files.map((file, fileIndex) => (
              <li key={file.path}>
                <button
                  className={
                    fileIndex === selectedFileIndex
                      ? "outline-link outline-link--active"
                      : "outline-link"
                  }
                  onClick={() => onSelectFile(fileIndex)}
                  type="button"
                >
                  {formatFileLabel(file.path)}
                </button>
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
