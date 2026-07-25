"use client";

import { useMemo, useState } from "react";
import { GroupCard } from "./group-card";
import { MarkdownInline, MarkdownText } from "./markdown-text";
import type { OutlineState } from "./outline-state";
import styles from "./file-list.module.css";
import type { BreadcrumbCrumb, ReaderNode } from "../lib/presenter";
import {
  buildFileBrowserEntries,
  buildNodeBreadcrumb,
  formatDirectoryLabel,
  makeFileAnchor,
  makeGroupAnchor,
  parentDirectory,
} from "../lib/presenter";
import type { DirectoryView, FileView, GroupView } from "../lib/types";
import type { PageView } from "../lib/types";

const NARROW_OUTLINE_MEDIA_QUERY = "(max-width: 860px)";

/** Props for coordinating outline navigation and selected document state. */
interface FileListProps {
  /** Collection title, used as the breadcrumb root and top-level outline label. */
  collectionTitle?: string;
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
  /** Called to select a node in the linear reading order (Next/Prev, crumbs, header). */
  onSelectNode: (nodeIndex: number) => void;
  /** Current outline visibility mode. */
  outlineState: OutlineState;
  /** Linear reading order of divider + file nodes. */
  readerNodes: ReaderNode[];
  /** Index of the file currently shown, or -1 when a divider is shown. */
  selectedFileIndex: number;
  /** Index of the current node in the reading order. */
  selectedNodeIndex: number;
}

/** Renders the collection outline beside the selected file's group cards. */
export function FileList({
  collectionTitle,
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
  onSelectNode,
  outlineState,
  readerNodes,
  selectedFileIndex,
  selectedNodeIndex,
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

  const selectedNode = readerNodes[selectedNodeIndex] ?? readerNodes[0];
  const isDivider = selectedNode?.kind === "divider";
  const activeFileIndex =
    selectedNode?.kind === "file" ? selectedNode.fileIndex : -1;
  const selectedFile = isDivider ? undefined : files[selectedFileIndex];
  const entries = buildFileBrowserEntries(files, directories, currentDirectory);

  const rootLabel = collectionTitle?.trim() || "Contents";
  const parentPath = currentDirectory ? parentDirectory(currentDirectory) : "";
  // Where the "up" control leads: the parent section, or the collection root.
  const upLabel = currentDirectory
    ? parentPath
      ? formatDirectoryLabel(directories, parentPath)
      : rootLabel
    : "";
  // "You are here": the current section title.
  const sectionLabel = currentDirectory
    ? formatDirectoryLabel(directories, currentDirectory)
    : rootLabel;
  // The section-title header is active when its own divider page is showing.
  const isSectionDividerActive =
    isDivider && selectedNode.directory === currentDirectory;
  // The cover link is active when the collection's own title page is showing.
  const isCoverActive = isDivider && selectedNode.directory === "";

  const breadcrumb = selectedNode
    ? buildNodeBreadcrumb(directories, selectedNode, rootLabel)
    : [];

  // Prev/Next walk the whole linear reading order (dividers and files alike).
  const previousNodeIndex =
    selectedNodeIndex > 0 ? selectedNodeIndex - 1 : null;
  const nextNodeIndex =
    selectedNodeIndex < readerNodes.length - 1 ? selectedNodeIndex + 1 : null;

  const selectNodeAndScroll = (nodeIndex: number) => {
    onSelectNode(nodeIndex);

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
        <button
          className={
            isCoverActive
              ? `${styles.outlineCover} ${styles.outlineCoverActive}`
              : styles.outlineCover
          }
          onClick={() => {
            onNavigateDirectory("");
            closeOutlineOnNarrowViewport();
          }}
          type="button"
        >
          <CoverMark />
          <span className={styles.outlineCoverText}>{rootLabel}</span>
        </button>
        {currentDirectory ? (
          <div className={styles.outlineHeader}>
            {parentPath ? (
              <button
                className={styles.outlineUp}
                onClick={() => onNavigateDirectory(parentPath)}
                type="button"
              >
                <span aria-hidden="true" className={styles.outlineUpChevron} />
                <span className={styles.outlineUpText}>{upLabel}</span>
              </button>
            ) : null}
            <button
              className={
                isSectionDividerActive
                  ? `${styles.outlineSectionTitle} ${styles.outlineSectionTitleActive}`
                  : styles.outlineSectionTitle
              }
              onClick={() => {
                onNavigateDirectory(currentDirectory);
                closeOutlineOnNarrowViewport();
              }}
              type="button"
            >
              {sectionLabel}
            </button>
          </div>
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
                    <ChapterMark />
                    <span className={styles.outlineLinkLabel}>
                      {entry.label}
                    </span>
                    <span
                      aria-hidden="true"
                      className={styles.outlineLinkChevron}
                    />
                  </button>
                ) : (
                  <button
                    className={
                      entry.fileIndex === activeFileIndex
                        ? `${styles.outlineLink} ${styles.outlineLinkFile} ${styles.outlineLinkActive}`
                        : `${styles.outlineLink} ${styles.outlineLinkFile}`
                    }
                    onClick={() => {
                      onSelectFile(entry.fileIndex);
                      closeOutlineOnNarrowViewport();
                    }}
                    type="button"
                  >
                    <span
                      aria-hidden="true"
                      className={styles.outlineFileDot}
                    />
                    <span className={styles.outlineLinkLabel}>
                      {entry.label}
                    </span>
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
          id={selectedFile ? makeFileAnchor(selectedFile.path) : undefined}
          key={
            isDivider || !selectedFile
              ? `divider:${selectedNode?.directory ?? ""}`
              : selectedFile.path
          }
        >
          {isDivider || !selectedFile ? (
            <DividerPage
              breadcrumb={breadcrumb}
              onNavigate={(directory) => onNavigateDirectory(directory)}
              title={selectedNode?.title ?? rootLabel}
            />
          ) : (
            <div className={styles.groupStream}>
              <Breadcrumb
                crumbs={breadcrumb}
                onNavigate={(directory) => onNavigateDirectory(directory)}
              />
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
                                  handleReferenceClick(
                                    anchorId,
                                    nextReferenceKey,
                                  )
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
          )}
          {readerNodes.length > 1 ? (
            <PageNavigation
              nextLabel={
                nextNodeIndex === null
                  ? null
                  : (readerNodes[nextNodeIndex]?.title ?? null)
              }
              onNext={() => {
                if (nextNodeIndex !== null) {
                  selectNodeAndScroll(nextNodeIndex);
                }
              }}
              onPrevious={() => {
                if (previousNodeIndex !== null) {
                  selectNodeAndScroll(previousNodeIndex);
                }
              }}
              previousLabel={
                previousNodeIndex === null
                  ? null
                  : (readerNodes[previousNodeIndex]?.title ?? null)
              }
            />
          ) : null}
        </article>
      </section>
    </div>
  );
}

/** Sleek breadcrumb trail shown above the content, marking the current location. */
function Breadcrumb({
  crumbs,
  onNavigate,
}: {
  crumbs: BreadcrumbCrumb[];
  onNavigate: (directory: string) => void;
}) {
  if (crumbs.length <= 1) {
    return null;
  }

  return (
    <nav aria-label="Breadcrumb" className={styles.breadcrumb}>
      <ol className={styles.breadcrumbList}>
        {crumbs.map((crumb, index) => {
          const isLast = index === crumbs.length - 1;

          return (
            <li
              className={styles.breadcrumbItem}
              key={`${crumb.label}-${index}`}
            >
              {crumb.directory !== null && !isLast ? (
                <button
                  className={styles.breadcrumbLink}
                  onClick={() => onNavigate(crumb.directory as string)}
                  type="button"
                >
                  {crumb.label}
                </button>
              ) : (
                <span
                  aria-current={isLast ? "page" : undefined}
                  className={styles.breadcrumbCurrent}
                >
                  {crumb.label}
                </span>
              )}
              {isLast ? null : (
                <span
                  aria-hidden="true"
                  className={styles.breadcrumbSeparator}
                />
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
}

/** Small "cover" glyph (stacked pages) for the link back to the collection start. */
function CoverMark() {
  return (
    <svg
      aria-hidden="true"
      className={styles.outlineCoverMark}
      fill="none"
      viewBox="0 0 16 16"
    >
      <path
        d="M2.75 3.4h6.7c.83 0 1.5.67 1.5 1.5v7.7h-6.7c-.83 0-1.5-.67-1.5-1.5V3.4Z"
        stroke="currentColor"
        strokeLinejoin="round"
        strokeWidth="1.2"
      />
      <path
        d="M5.25 2.4h6.7c.83 0 1.5.67 1.5 1.5v7.7"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.2"
      />
    </svg>
  );
}

/**
 * Abstract bookmark glyph marking a section (a "chapter") the reader can open.
 * Deliberately not a folder: it reads as a place-marker in a book.
 */
function ChapterMark() {
  return (
    <svg
      aria-hidden="true"
      className={styles.outlineChapterMark}
      fill="none"
      viewBox="0 0 16 16"
    >
      <path
        d="M4.75 3.1a.85.85 0 0 1 .85-.85h4.8a.85.85 0 0 1 .85.85v10.05L8 10.9l-3.25 2.25V3.1Z"
        stroke="currentColor"
        strokeLinejoin="round"
        strokeWidth="1.3"
      />
    </svg>
  );
}

/**
 * Chapter-opener header shown when the reader lands on a section's page. It
 * announces the move into a new part of the collection, like a book divider.
 */
/**
 * A divider page: the centered title of the collection root or a directory,
 * shown as its own stop in the reading order — like a book's part divider.
 */
function DividerPage({
  breadcrumb,
  onNavigate,
  title,
}: {
  breadcrumb: BreadcrumbCrumb[];
  onNavigate: (directory: string) => void;
  title: string;
}) {
  return (
    <div className={styles.dividerPage}>
      <Breadcrumb crumbs={breadcrumb} onNavigate={onNavigate} />
      <div className={styles.dividerCenter}>
        <h1 className={styles.dividerTitle}>{title}</h1>
        <span aria-hidden="true" className={styles.dividerRule} />
      </div>
    </div>
  );
}

function PageLoadingState() {
  return (
    <div
      aria-label="Loading page"
      className={styles.pageSkeleton}
      role="status"
    >
      <div aria-hidden="true" className={styles.skeletonTextBlock}>
        <span className={`${styles.skeletonLine} ${styles.skeletonLineLong}`} />
        <span
          className={`${styles.skeletonLine} ${styles.skeletonLineShort}`}
        />
      </div>
      <div aria-hidden="true" className={styles.skeletonCard}>
        <span
          className={`${styles.skeletonLine} ${styles.skeletonLineTitle}`}
        />
        <span className={styles.skeletonDivider} />
        <span
          className={`${styles.skeletonLine} ${styles.skeletonLineMedium}`}
        />
        <span className={`${styles.skeletonLine} ${styles.skeletonLineLong}`} />
        <span
          className={`${styles.skeletonLine} ${styles.skeletonLineShort}`}
        />
      </div>
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
    <div
      aria-label="Loading definition"
      className={styles.definitionSkeleton}
      role="status"
    >
      <div aria-hidden="true" className={styles.skeletonCard}>
        <span
          className={`${styles.skeletonLine} ${styles.skeletonLineTitle}`}
        />
        <span className={styles.skeletonDivider} />
        <span
          className={`${styles.skeletonLine} ${styles.skeletonLineMedium}`}
        />
        <span className={`${styles.skeletonLine} ${styles.skeletonLineLong}`} />
        <span
          className={`${styles.skeletonLine} ${styles.skeletonLineShort}`}
        />
      </div>
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
  nextLabel,
  onNext,
  onPrevious,
  previousLabel,
}: {
  nextLabel: string | null;
  onNext: () => void;
  onPrevious: () => void;
  previousLabel: string | null;
}) {
  return (
    <nav aria-label="Page navigation" className={styles.pageNavigation}>
      {previousLabel ? (
        <button
          aria-label={`Previous: ${previousLabel}`}
          className={styles.pageNavButton}
          onClick={onPrevious}
          title={`Previous: ${previousLabel}`}
          type="button"
        >
          <PageNavigationIcon direction="previous" />
          <span className={styles.pageNavText}>
            <span className={styles.pageNavLabel}>Previous</span>
            <span className={styles.pageNavTitle}>{previousLabel}</span>
          </span>
        </button>
      ) : null}
      {nextLabel ? (
        <button
          aria-label={`Next: ${nextLabel}`}
          className={`${styles.pageNavButton} ${styles.pageNavButtonNext}`}
          onClick={onNext}
          title={`Next: ${nextLabel}`}
          type="button"
        >
          <span className={styles.pageNavText}>
            <span className={styles.pageNavLabel}>Next</span>
            <span className={styles.pageNavTitle}>{nextLabel}</span>
          </span>
          <PageNavigationIcon direction="next" />
        </button>
      ) : null}
    </nav>
  );
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
