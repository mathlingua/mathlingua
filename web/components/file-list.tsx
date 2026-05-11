import { GroupCard } from "./group-card";
import {
  formatFileSummary,
  formatFileTitle,
  makeGroupAnchor,
} from "../lib/presenter";
import { FileView } from "../lib/types";

type FileListProps = {
  files: FileView[];
};

export function FileList({ files }: FileListProps) {
  if (files.length === 0) {
    return (
      <section className="empty-state">
        <h2>No renderable files found</h2>
        <p>Add Mathlingua files under your collection content directory.</p>
      </section>
    );
  }

  return (
    <div className="reader-layout">
      <aside className="outline-panel">
        <p className="outline-title">Outline</p>
        <nav>
          <ul className="outline-list">
            {files.map((file, index) => (
              <li key={file.path}>
                <a href={`#file-${index}`}>{formatFileTitle(file)}</a>
              </li>
            ))}
          </ul>
        </nav>
      </aside>
      <section className="chapter-stack">
        {files.map((file, fileIndex) => (
          <article className="chapter-panel" id={`file-${fileIndex}`} key={file.path}>
            <header className="chapter-header">
              <p className="chapter-kicker">{file.path}</p>
              <h2>{formatFileTitle(file)}</h2>
              <p className="chapter-summary">{formatFileSummary(file)}</p>
            </header>
            <div className="chapter-groups">
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
