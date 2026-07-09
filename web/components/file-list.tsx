"use client";

import { useMemo, useState } from "react";
import { GroupCard } from "./group-card";
import { MarkdownInline, MarkdownText } from "./markdown-text";
import type { OutlineState } from "./outline-state";
import styles from "./file-list.module.css";
import {
  buildFileBrowserEntries,
  formatFileLabel,
  formatPathSegment,
  makeFileAnchor,
  makeGroupAnchor,
  parentDirectory,
} from "../lib/presenter";
import type { DirectoryView, FileView, GroupView } from "../lib/types";
import type { PageView } from "../lib/types";

const NARROW_OUTLINE_MEDIA_QUERY = "(max-width: 860px)";

/** Props for coordinating outline navigation and selected document state. */
interface FileListProps {
  /** Directory currently shown by the outline browser. */
  currentDirectory: string;
  /** Static export definition key to item id map. */
  definitionItemIds?: Record<string, string>;
  /** Renderable directories in the collection. */
  directories: DirectoryView[];
  /** Renderable files in the collection. */
  files: FileView[];
  /** True while the selected static-export page payload is being loaded. */
  isSelectedFileLoading?: boolean;
  /** Static-export page load error for the selected file, if one occurred. */
  loadError?: string;
  /** Definition cards loaded lazily by reference key. */
  loadedDefinitions?: Record<string, GroupView>;
  /** Called when the outline drawer should be closed. */
  onCloseOutline: () => void;
  /** Called when a reference key should be loaded lazily. */
  onLoadDefinition?: (referenceKey: string) => void;
  /** Called when the user drills into or backs out of an outline directory. */
  onNavigateDirectory: (directory: string) => void;
  /** Called when the user selects a file from the outline. */
  onSelectFile: (fileIndex: number) => void;
  /** Current outline visibility mode. */
  outlineState: OutlineState;
  /** Index of the file currently shown in the document stream. */
  selectedFileIndex: number;
}

/** Renders the collection outline beside the selected file's group cards. */
export function FileList({
  currentDirectory,
  definitionItemIds,
  directories,
  files,
  isSelectedFileLoading = false,
  loadError,
  loadedDefinitions,
  onCloseOutline,
  onLoadDefinition,
  onNavigateDirectory,
  onSelectFile,
  outlineState,
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

  const activeFileIndex = files[selectedFileIndex] ? selectedFileIndex : 0;
  const selectedFile = files[activeFileIndex] ?? files[0];
  const entries = buildFileBrowserEntries(files, directories, currentDirectory);
  const previousFileIndex = activeFileIndex > 0 ? activeFileIndex - 1 : null;
  const nextFileIndex =
    activeFileIndex < files.length - 1 ? activeFileIndex + 1 : null;

  const handleSelectPage = (fileIndex: number) => {
    onSelectFile(fileIndex);

    if (typeof window !== "undefined") {
      window.requestAnimationFrame(() => {
        window.scrollTo({ top: 0, behavior: "smooth" });
      });
    }
  };

  const closeOutlineOnNarrowViewport = () => {
    if (
      typeof window !== "undefined" &&
      window.matchMedia(NARROW_OUTLINE_MEDIA_QUERY).matches
    ) {
      onCloseOutline();
    }
  };

  const handleReferenceClick = (rootAnchorId: string, referenceKey: string) => {
    if (
      !definitionIndex.has(referenceKey) &&
      !loadedDefinitions?.[referenceKey] &&
      !definitionItemIds?.[referenceKey]
    ) {
      return;
    }

    onLoadDefinition?.(referenceKey);

    setDefinitionTrails((current) => {
      const existingTrail = current[rootAnchorId] ?? [];
      const nextTrail = [
        ...existingTrail.filter((key) => key !== referenceKey),
        referenceKey,
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
        outlineState === "auto"
          ? `${styles.readerLayout} ${styles.outlineAuto}`
          : outlineState === "open"
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
                    onClick={() => {
                      onNavigateDirectory(entry.path);
                      closeOutlineOnNarrowViewport();
                    }}
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
                      entry.fileIndex === activeFileIndex
                        ? `${styles.outlineLink} ${styles.outlineLinkActive}`
                        : styles.outlineLink
                    }
                    onClick={() => {
                      onSelectFile(entry.fileIndex);
                      closeOutlineOnNarrowViewport();
                    }}
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
            {loadError ? <PageLoadError message={loadError} /> : null}
            {isSelectedFileLoading ? <PageLoadingState /> : null}
            {selectedFile.items.map((item, itemIndex) => {
              const fallbackKey = `${activeFileIndex}-${itemIndex}`;
              const anchorId = makeGroupAnchor(item, fallbackKey);
              const trail = definitionTrails[anchorId] ?? [];
              const itemKey =
                item.id || `${selectedFile.path}-${item.kind}-${itemIndex}`;

              if (item.page) {
                return (
                  <PageItem
                    anchorId={anchorId}
                    key={itemKey}
                    page={item.page}
                  />
                );
              }

              return (
                <div className={styles.definitionStack} key={itemKey}>
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
                        const definition =
                          definitionIndex.get(referenceKey)?.group ??
                          loadedDefinitions?.[referenceKey];

                        if (!definition) {
                          return (
                            <LoadingDefinition
                              key={`${referenceKey}-${trailIndex}`}
                            />
                          );
                        }

                        return (
                          <div
                            className={styles.definitionTrailItem}
                            key={`${referenceKey}-${trailIndex}`}
                          >
                            <GroupCard
                              anchorId={`${makeGroupAnchor(
                                definition,
                                `${anchorId}-definition-${trailIndex}`,
                              )}-definition-${trailIndex}`}
                              group={definition}
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
          {files.length > 1 ? (
            <PageNavigation
              nextFile={nextFileIndex === null ? null : files[nextFileIndex]}
              onNext={() => {
                if (nextFileIndex !== null) {
                  handleSelectPage(nextFileIndex);
                }
              }}
              onPrevious={() => {
                if (previousFileIndex !== null) {
                  handleSelectPage(previousFileIndex);
                }
              }}
              previousFile={
                previousFileIndex === null ? null : files[previousFileIndex]
              }
            />
          ) : null}
        </article>
      </section>
    </div>
  );
}

function PageLoadingState() {
  return (
    <div className={styles.loadingState} role="status">
      Loading page...
    </div>
  );
}

function PageLoadError({ message }: { message: string }) {
  return (
    <div className={styles.loadError} role="alert">
      Could not load page data: {message}
    </div>
  );
}

function LoadingDefinition() {
  return (
    <div className={styles.loadingDefinition} role="status">
      Loading definition...
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

function PageNavigation({
  nextFile,
  onNext,
  onPrevious,
  previousFile,
}: {
  nextFile: FileView | null;
  onNext: () => void;
  onPrevious: () => void;
  previousFile: FileView | null;
}) {
  return (
    <nav aria-label="Page navigation" className={styles.pageNavigation}>
      {previousFile ? (
        <button
          aria-label={`Previous page: ${pageNavigationLabel(previousFile)}`}
          className={styles.pageNavButton}
          onClick={onPrevious}
          title={`Previous: ${pageNavigationLabel(previousFile)}`}
          type="button"
        >
          <PageNavigationIcon direction="previous" />
          <span className={styles.pageNavText}>
            <span className={styles.pageNavTitle}>
              {pageNavigationLabel(previousFile)}
            </span>
          </span>
        </button>
      ) : null}
      {nextFile ? (
        <button
          aria-label={`Next page: ${pageNavigationLabel(nextFile)}`}
          className={`${styles.pageNavButton} ${styles.pageNavButtonNext}`}
          onClick={onNext}
          title={`Next: ${pageNavigationLabel(nextFile)}`}
          type="button"
        >
          <span className={styles.pageNavText}>
            <span className={styles.pageNavTitle}>
              {pageNavigationLabel(nextFile)}
            </span>
          </span>
          <PageNavigationIcon direction="next" />
        </button>
      ) : null}
    </nav>
  );
}

function pageNavigationLabel(file: FileView): string {
  return file.title ?? formatFileLabel(file.path);
}

function PageNavigationIcon({ direction }: { direction: "next" | "previous" }) {
  const path = direction === "next" ? "M8 5l7 7-7 7" : "M16 5l-7 7 7 7";

  return (
    <svg
      aria-hidden="true"
      className={styles.pageNavIcon}
      focusable="false"
      viewBox="0 0 24 24"
    >
      <path d={path} />
    </svg>
  );
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
