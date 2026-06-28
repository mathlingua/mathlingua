"use client";

import { useMemo, useState } from "react";
import { GroupCard } from "./group-card";
import { MarkdownInline, MarkdownText } from "./markdown-text";
import styles from "./file-list.module.css";
import {
  buildFileBrowserEntries,
  formatPathSegment,
  makeFileAnchor,
  makeGroupAnchor,
  parentDirectory,
} from "../lib/presenter";
import type { DirectoryView, FileView, GroupView } from "../lib/types";
import type { PageView } from "../lib/types";

/** Props for coordinating outline navigation and selected document state. */
interface FileListProps {
  /** Directory currently shown by the outline browser. */
  currentDirectory: string;
  /** Renderable directories in the collection. */
  directories: DirectoryView[];
  /** Renderable files in the collection. */
  files: FileView[];
  /** Whether the outline panel is visible. */
  isOutlineOpen: boolean;
  /** Called when the user drills into or backs out of an outline directory. */
  onNavigateDirectory: (directory: string) => void;
  /** Called when the user selects a file from the outline. */
  onSelectFile: (fileIndex: number) => void;
  /** Index of the file currently shown in the document stream. */
  selectedFileIndex: number;
}

/** Renders the collection outline beside the selected file's group cards. */
export function FileList({
  currentDirectory,
  directories,
  files,
  isOutlineOpen,
  onNavigateDirectory,
  onSelectFile,
  selectedFileIndex,
}: FileListProps) {
  const [definitionTrails, setDefinitionTrails] = useState<
    Record<string, string[]>
  >({});
  const definitionIndex = useMemo(() => buildDefinitionIndex(files), [files]);

  if (files.length === 0) {
    return (
      <section className={styles.emptyState}>
        <h2>No renderable files found</h2>
        <p>Add Mathlingua files under your collection content directory.</p>
      </section>
    );
  }

  const selectedFile = files[selectedFileIndex] ?? files[0];
  const entries = buildFileBrowserEntries(files, directories, currentDirectory);

  const handleReferenceClick = (rootAnchorId: string, referenceKey: string) => {
    if (!definitionIndex.has(referenceKey)) {
      return;
    }

    setDefinitionTrails((current) => {
      const existingTrail = current[rootAnchorId] ?? [];
      const nextTrail = [
        referenceKey,
        ...existingTrail.filter((key) => key !== referenceKey),
      ];

      return {
        ...current,
        [rootAnchorId]: nextTrail,
      };
    });
  };

  const handleCloseDefinition = (rootAnchorId: string, index: number) => {
    setDefinitionTrails((current) => {
      const nextTrail = (current[rootAnchorId] ?? []).filter(
        (_, trailIndex) => trailIndex !== index,
      );
      const next = { ...current };

      if (nextTrail.length === 0) {
        delete next[rootAnchorId];
      } else {
        next[rootAnchorId] = nextTrail;
      }

      return next;
    });
  };

  const handleCloseDefinitionTrail = (rootAnchorId: string) => {
    setDefinitionTrails((current) => {
      if (!current[rootAnchorId]) {
        return current;
      }

      const next = { ...current };
      delete next[rootAnchorId];
      return next;
    });
  };

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
                    {entry.label}
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
            {selectedFile.items.map((item, itemIndex) => {
              const anchorId = makeGroupAnchor(selectedFileIndex, itemIndex);
              const trail = definitionTrails[anchorId] ?? [];

              if (item.page) {
                return (
                  <PageItem
                    anchorId={anchorId}
                    key={`${selectedFile.path}-${item.kind}-${itemIndex}`}
                    page={item.page}
                  />
                );
              }

              return (
                <div
                  className={styles.definitionStack}
                  key={`${selectedFile.path}-${item.kind}-${itemIndex}`}
                >
                  <GroupCard
                    anchorId={anchorId}
                    group={item}
                    onReferenceClick={(referenceKey) =>
                      handleReferenceClick(anchorId, referenceKey)
                    }
                  />
                  {trail.length > 0 ? (
                    <div className={styles.definitionTrail}>
                      <button
                        aria-label="Close all definitions"
                        className={styles.definitionTrailClose}
                        onClick={() => handleCloseDefinitionTrail(anchorId)}
                        title="Close all definitions"
                        type="button"
                      >
                        <DefinitionTrailCloseIcon />
                      </button>
                      {trail.map((referenceKey, trailIndex) => {
                        const definition = definitionIndex.get(referenceKey);

                        if (!definition) {
                          return null;
                        }

                        return (
                          <div
                            className={styles.definitionTrailItem}
                            key={`${referenceKey}-${trailIndex}`}
                          >
                            <GroupCard
                              anchorId={`${anchorId}-definition-${trailIndex}`}
                              group={definition.group}
                              onClose={() =>
                                handleCloseDefinition(anchorId, trailIndex)
                              }
                              onReferenceClick={(nextReferenceKey) =>
                                handleReferenceClick(anchorId, nextReferenceKey)
                              }
                            />
                          </div>
                        );
                      })}
                    </div>
                  ) : null}
                </div>
              );
            })}
          </div>
        </article>
      </section>
    </div>
  );
}

function PageItem({ anchorId, page }: { anchorId: string; page: PageView }) {
  if (page.kind === "Text") {
    return (
      <section
        className={`${styles.pageItem} ${styles.pageText}`}
        id={anchorId}
      >
        <MarkdownText text={page.text} />
      </section>
    );
  }

  return (
    <section
      className={`${styles.pageItem} ${styles[`page${page.kind}`] ?? ""}`}
      id={anchorId}
    >
      <PageHeading kind={page.kind} text={page.text} />
    </section>
  );
}

function PageHeading({ kind, text }: { kind: string; text: string }) {
  switch (kind) {
    case "Title":
      return (
        <h1>
          <MarkdownInline text={text} />
        </h1>
      );
    case "SectionTitle":
      return (
        <h2>
          <MarkdownInline text={text} />
        </h2>
      );
    case "SubsectionTitle":
      return (
        <h3>
          <MarkdownInline text={text} />
        </h3>
      );
    default:
      return (
        <h3>
          <MarkdownInline text={text} />
        </h3>
      );
  }
}

interface DefinitionIndexEntry {
  group: GroupView;
}

function buildDefinitionIndex(
  files: FileView[],
): Map<string, DefinitionIndexEntry> {
  const definitions = new Map<string, DefinitionIndexEntry>();

  for (const file of files) {
    for (const group of file.items) {
      for (const key of group.definition_keys ?? []) {
        if (!definitions.has(key)) {
          definitions.set(key, { group });
        }
      }
    }
  }

  return definitions;
}

function DefinitionTrailCloseIcon() {
  return (
    <svg
      aria-hidden="true"
      className={styles.definitionTrailCloseIcon}
      focusable="false"
      viewBox="0 0 24 24"
    >
      <path d="M6 6l12 12" />
      <path d="M18 6 6 18" />
    </svg>
  );
}
