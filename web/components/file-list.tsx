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
};

export function FileList({ files, isOutlineOpen }: FileListProps) {
  if (files.length === 0) {
    return (
      <section className="empty-state">
        <h2>No renderable files found</h2>
        <p>Add Mathlingua files under your collection content directory.</p>
      </section>
    );
  }

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
                <a href={`#${makeFileAnchor(fileIndex)}`}>
                  {formatFileLabel(file.path)}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      </aside>
      <section className="document-stream">
        {files.map((file, fileIndex) => (
          <article
            className="file-section"
            id={makeFileAnchor(fileIndex)}
            key={file.path}
          >
            <div className="group-stream">
              {file.items.map((item, itemIndex) => (
                <GroupCard
                  anchorId={makeGroupAnchor(fileIndex, itemIndex)}
                  group={item}
                  key={`${file.path}-${item.kind}-${itemIndex}`}
                />
              ))}
            </div>
          </article>
        ))}
      </section>
    </div>
  );
}
