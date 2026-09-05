"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { FileList } from "./file-list";
import type { OutlineState } from "./outline-state";
import { ViewerChrome } from "./viewer-chrome";
import {
  DEFAULT_VIEWER_THEME,
  VIEWER_THEME_STORAGE_KEY,
  applyViewerTheme,
  isViewerTheme,
  type ViewerTheme,
} from "./viewer-theme";
import styles from "./viewer-shell.module.css";
import {
  CollectionManifest,
  CollectionView,
  DirectoryView,
  FileManifest,
  FileView,
  GroupView,
  PageData,
} from "../lib/types";
import {
  buildReaderNodes,
  nodeIndexForDirectory,
  nodeIndexForFile,
  readerNodeRoute,
  resolveReaderNodeIndex,
} from "../lib/presenter";

const NARROW_OUTLINE_MEDIA_QUERY = "(max-width: 860px)";
const SHOW_TYPES_STORAGE_KEY = "mlg-view-show-types";

/** Props for the client-side viewer state container. */
interface ViewerShellProps {
  /** Full collection view supplied by `mlg view` development mode. */
  initialCollection?: CollectionView;
  /** Live collection JSON supplied by the embedded `mlg view` server. */
  collectionDataPath?: string;
  /** Lightweight static export manifest supplied at build time. */
  initialManifest?: CollectionManifest;
  /** Initial browser pathname supplied by the server route. */
  initialPathname: string;
  /** URL path prefix used by static project-page hosting. */
  routeBasePath?: string;
  /** Static data root used by `mlg export`; omitted in development mode. */
  staticDataBasePath?: string;
}

/** Owns browser history, selected file, outline directory, and chrome state. */
export function ViewerShell({
  initialCollection,
  collectionDataPath,
  initialManifest,
  initialPathname,
  routeBasePath: routeBasePathProp = "",
  staticDataBasePath,
}: ViewerShellProps) {
  const routeBasePath = normalizeClientBasePath(routeBasePathProp);
  const useTrailingSlashRoutes = Boolean(staticDataBasePath);
  const [manifest, setManifest] = useState<CollectionManifest | null>(() =>
    initialCollection
      ? manifestFromCollection(initialCollection)
      : (initialManifest ?? null),
  );
  const [loadedFiles, setLoadedFiles] = useState<Record<string, FileView>>(
    () =>
      initialCollection
        ? Object.fromEntries(
            initialCollection.files.map((file) => [file.path, file]),
          )
        : {},
  );
  const [loadingFilePaths, setLoadingFilePaths] = useState<
    Record<string, boolean>
  >({});
  const [fileLoadErrors, setFileLoadErrors] = useState<Record<string, string>>(
    {},
  );
  const [loadedDefinitions, setLoadedDefinitions] = useState<
    Record<string, GroupView>
  >({});
  const [loadingDefinitionKeys, setLoadingDefinitionKeys] = useState<
    Record<string, boolean>
  >({});
  const warmedFilePaths = useRef<Set<string>>(new Set());
  const directories = manifest?.directories ?? [];
  const fileEntries = manifest?.files ?? [];
  const files = useMemo(
    () =>
      fileEntries.map(
        (file) => loadedFiles[file.path] ?? fileViewFromManifest(file),
      ),
    [fileEntries, loadedFiles],
  );
  const collectionTitle = manifest?.title ?? "";
  const readerNodes = useMemo(
    () => buildReaderNodes(fileEntries, directories, collectionTitle),
    [fileEntries, directories, collectionTitle],
  );
  const [outlineState, setOutlineState] = useState<OutlineState>("auto");
  const [selectedNodeIndex, setSelectedNodeIndex] = useState(() =>
    resolveReaderNodeIndex(
      stripRouteBasePath(initialPathname, routeBasePath),
      readerNodes,
    ),
  );
  const [browsedDirectory, setBrowsedDirectory] = useState<string | null>(null);
  const [theme, setTheme] = useState<ViewerTheme>(DEFAULT_VIEWER_THEME);
  const [showTypes, setShowTypes] = useState(false);

  const selectedNode = readerNodes[selectedNodeIndex] ?? readerNodes[0];
  const currentDirectory = browsedDirectory ?? selectedNode?.directory ?? "";
  const selectedFileIndex =
    selectedNode?.kind === "file" ? selectedNode.fileIndex : -1;

  const loadFileIntoCache = useCallback(
    (file: FileManifest, options: { recordError: boolean }) => {
      if (!staticDataBasePath || !manifest) {
        return;
      }

      setLoadingFilePaths((current) => ({ ...current, [file.path]: true }));

      loadStaticFile(staticDataBasePath, manifest, file)
        .then((loadedFile) => {
          setLoadedFiles((current) => ({
            ...current,
            [loadedFile.path]: loadedFile,
          }));
        })
        .catch((error) => {
          console.error(`Failed to load MathLingua page ${file.path}`, error);
          if (options.recordError) {
            setFileLoadErrors((current) => ({
              ...current,
              [file.path]: readableErrorMessage(error),
            }));
          }
        })
        .finally(() => {
          setLoadingFilePaths((current) => ({
            ...current,
            [file.path]: false,
          }));
        });
    },
    [staticDataBasePath, manifest],
  );

  useEffect(() => {
    if (!collectionDataPath || manifest) {
      return;
    }

    let cancelled = false;
    fetchJson<CollectionView>(collectionDataPath)
      .then((collection) => {
        if (cancelled) {
          return;
        }

        setManifest(manifestFromCollection(collection));
        setLoadedFiles(
          Object.fromEntries(collection.files.map((file) => [file.path, file])),
        );
      })
      .catch((error) => {
        console.error("Failed to load MathLingua collection", error);
      });

    return () => {
      cancelled = true;
    };
  }, [collectionDataPath, manifest]);

  useEffect(() => {
    if (!staticDataBasePath || manifest) {
      return;
    }

    let cancelled = false;
    fetchJson<CollectionManifest>(
      joinDataPath(staticDataBasePath, "manifest.json"),
    )
      .then((nextManifest) => {
        if (!cancelled) {
          setManifest(nextManifest);
        }
      })
      .catch((error) => {
        console.error("Failed to load MathLingua export manifest", error);
      });

    return () => {
      cancelled = true;
    };
  }, [staticDataBasePath, manifest]);

  useEffect(() => {
    const title = manifest?.title.trim();
    document.title = title || "MathLingua Viewer";
  }, [manifest?.title]);

  useEffect(() => {
    if (!staticDataBasePath || !manifest) {
      return;
    }

    const file = manifest.files[selectedFileIndex];
    if (
      !file ||
      loadedFiles[file.path] ||
      loadingFilePaths[file.path] ||
      fileLoadErrors[file.path]
    ) {
      return;
    }

    loadFileIntoCache(file, { recordError: true });
  }, [
    staticDataBasePath,
    manifest,
    selectedFileIndex,
    loadedFiles,
    loadingFilePaths,
    fileLoadErrors,
    loadFileIntoCache,
  ]);

  useEffect(() => {
    if (!staticDataBasePath || !manifest) {
      return;
    }

    const currentFile = manifest.files[selectedFileIndex];
    if (
      currentFile &&
      !loadedFiles[currentFile.path] &&
      !fileLoadErrors[currentFile.path]
    ) {
      return;
    }

    const nextFile = manifest.files[selectedFileIndex + 1];
    if (
      !nextFile ||
      loadedFiles[nextFile.path] ||
      loadingFilePaths[nextFile.path] ||
      warmedFilePaths.current.has(nextFile.path)
    ) {
      return;
    }

    warmedFilePaths.current.add(nextFile.path);
    loadFileIntoCache(nextFile, { recordError: false });
  }, [
    staticDataBasePath,
    manifest,
    selectedFileIndex,
    loadedFiles,
    loadingFilePaths,
    fileLoadErrors,
    loadFileIntoCache,
  ]);

  useEffect(() => {
    const documentTheme = document.documentElement.dataset.theme;
    const storedTheme = readStoredTheme();
    const initialTheme = isViewerTheme(documentTheme)
      ? documentTheme
      : isViewerTheme(storedTheme)
        ? storedTheme
        : DEFAULT_VIEWER_THEME;

    setTheme(initialTheme);
    applyViewerTheme(initialTheme);
    setShowTypes(readStoredBoolean(SHOW_TYPES_STORAGE_KEY));
  }, []);

  useEffect(() => {
    const syncSelectedNodeFromPath = () => {
      setBrowsedDirectory(null);
      setSelectedNodeIndex(
        resolveReaderNodeIndex(
          stripRouteBasePath(window.location.pathname, routeBasePath),
          readerNodes,
        ),
      );
    };

    syncSelectedNodeFromPath();
    window.addEventListener("popstate", syncSelectedNodeFromPath);

    return () => {
      window.removeEventListener("popstate", syncSelectedNodeFromPath);
    };
  }, [readerNodes, routeBasePath]);

  const handleSelectNode = (nodeIndex: number) => {
    const node = readerNodes[nodeIndex];
    if (!node) {
      return;
    }

    setBrowsedDirectory(null);
    setSelectedNodeIndex(nodeIndex);
    window.history.pushState(
      null,
      "",
      withRouteBasePath(
        readerNodeRoute(node),
        routeBasePath,
        useTrailingSlashRoutes,
      ),
    );
  };

  const handleSelectFile = (fileIndex: number) => {
    const nodeIndex = nodeIndexForFile(readerNodes, fileIndex);
    if (nodeIndex >= 0) {
      handleSelectNode(nodeIndex);
    }
  };

  const handleNavigateDirectory = (directory: string) => {
    const nodeIndex = nodeIndexForDirectory(readerNodes, directory);
    if (nodeIndex >= 0) {
      handleSelectNode(nodeIndex);
    }
  };

  const handleToggleOutline = () => {
    setOutlineState((current) => {
      const isOpen =
        current === "auto"
          ? !window.matchMedia(NARROW_OUTLINE_MEDIA_QUERY).matches
          : current === "open";

      return isOpen ? "closed" : "open";
    });
  };

  const handleThemeChange = (nextTheme: ViewerTheme) => {
    setTheme(nextTheme);
    applyViewerTheme(nextTheme);
    writeStoredTheme(nextTheme);
  };

  const handleToggleTypes = () => {
    setShowTypes((current) => {
      const next = !current;
      writeStoredBoolean(SHOW_TYPES_STORAGE_KEY, next);
      return next;
    });
  };

  const handleLoadDefinition = useCallback(
    (referenceKey: string) => {
      if (
        !staticDataBasePath ||
        !manifest ||
        loadedDefinitions[referenceKey] ||
        loadingDefinitionKeys[referenceKey]
      ) {
        return;
      }

      const itemId = manifest.definitions[referenceKey];
      if (!itemId) {
        return;
      }

      setLoadingDefinitionKeys((current) => ({
        ...current,
        [referenceKey]: true,
      }));

      loadStaticItem(staticDataBasePath, manifest, itemId)
        .then((group) => {
          setLoadedDefinitions((current) => ({
            ...current,
            [referenceKey]: group,
          }));
        })
        .catch((error) => {
          console.error(
            `Failed to load MathLingua definition ${referenceKey}`,
            error,
          );
        })
        .finally(() => {
          setLoadingDefinitionKeys((current) => ({
            ...current,
            [referenceKey]: false,
          }));
        });
    },
    [staticDataBasePath, manifest, loadedDefinitions, loadingDefinitionKeys],
  );

  const selectedFilePath = fileEntries[selectedFileIndex]?.path;
  const selectedFileLoadError = selectedFilePath
    ? fileLoadErrors[selectedFilePath]
    : undefined;
  const isSelectedFileLoading = Boolean(
    staticDataBasePath &&
    selectedFilePath &&
    !selectedFileLoadError &&
    (loadingFilePaths[selectedFilePath] || !loadedFiles[selectedFilePath]),
  );

  return (
    <>
      <ViewerChrome
        onToggleOutline={handleToggleOutline}
        onToggleTypes={handleToggleTypes}
        onThemeChange={handleThemeChange}
        outlineState={outlineState}
        showTypes={showTypes}
        theme={theme}
      />
      <main className={styles.pageShell}>
        <FileList
          collectionPreface={manifest?.preface}
          collectionTitle={manifest?.title}
          currentDirectory={currentDirectory}
          definitionItemIds={manifest?.definitions}
          directories={directories}
          files={files}
          loadError={selectedFileLoadError}
          isSelectedFileLoading={isSelectedFileLoading}
          loadedDefinitions={loadedDefinitions}
          onBrowseDirectory={setBrowsedDirectory}
          onCloseOutline={() => setOutlineState("closed")}
          onLoadDefinition={handleLoadDefinition}
          onNavigateDirectory={handleNavigateDirectory}
          onSelectFile={handleSelectFile}
          onSelectNode={handleSelectNode}
          outlineState={outlineState}
          readerNodes={readerNodes}
          selectedFileIndex={selectedFileIndex}
          selectedNodeIndex={selectedNodeIndex}
          showTypes={showTypes}
        />
      </main>
    </>
  );
}

function manifestFromCollection(
  collection: CollectionView,
): CollectionManifest {
  const definitions: Record<string, string> = {};
  const items: Record<string, string> = {};

  for (const file of collection.files) {
    for (const group of file.items) {
      if (group.id) {
        items[group.id] = "";
      }

      for (const key of group.definition_keys ?? []) {
        if (!definitions[key] && group.id) {
          definitions[key] = group.id;
        }
      }
    }
  }

  return {
    schemaVersion: 1,
    title: collection.title,
    preface: collection.preface,
    directories: collection.directories,
    files: collection.files.map((file) => ({
      path: file.path,
      title: file.title,
      dataPath: "",
    })),
    definitions,
    items,
  };
}

function fileViewFromManifest(file: FileManifest): FileView {
  return {
    path: file.path,
    title: file.title,
    items: [],
  };
}

async function loadStaticFile(
  dataBasePath: string,
  manifest: CollectionManifest,
  file: FileManifest,
): Promise<FileView> {
  const page = await fetchJson<PageData>(
    joinDataPath(dataBasePath, file.dataPath),
  );
  const items = await Promise.all(
    page.itemIds.map((itemId) =>
      loadStaticItem(dataBasePath, manifest, itemId),
    ),
  );

  return {
    path: page.path,
    title: page.title,
    items,
  };
}

async function loadStaticItem(
  dataBasePath: string,
  manifest: CollectionManifest,
  itemId: string,
): Promise<GroupView> {
  const itemPath = manifest.items[itemId];
  if (!itemPath) {
    throw new Error(`No item path for ${itemId}`);
  }

  return fetchJson<GroupView>(joinDataPath(dataBasePath, itemPath));
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText}`);
  }

  return (await response.json()) as T;
}

function readableErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function joinDataPath(basePath: string, dataPath: string): string {
  return `${basePath.replace(/\/+$/, "")}/${dataPath.replace(/^\/+/, "")}`;
}

function normalizeClientBasePath(value: string): string {
  const trimmed = value.trim();
  if (!trimmed || trimmed === "/") {
    return "";
  }

  const withSlash = trimmed.startsWith("/") ? trimmed : `/${trimmed}`;
  return withSlash.replace(/\/+$/, "");
}

function stripRouteBasePath(pathname: string, basePath: string): string {
  if (!basePath || pathname === basePath) {
    return pathname === basePath ? "/" : pathname;
  }

  if (pathname.startsWith(`${basePath}/`)) {
    return pathname.slice(basePath.length) || "/";
  }

  return pathname;
}

function withRouteBasePath(
  pathname: string,
  basePath: string,
  useTrailingSlash: boolean,
): string {
  const routePath = useTrailingSlash
    ? withTrailingSlash(pathname)
    : withoutTrailingSlash(pathname);

  if (!basePath) {
    return routePath;
  }

  if (routePath === "/") {
    return useTrailingSlash ? `${basePath}/` : basePath;
  }

  return `${basePath}${routePath.startsWith("/") ? routePath : `/${routePath}`}`;
}

function withTrailingSlash(pathname: string): string {
  if (!pathname || pathname === "/") {
    return "/";
  }

  return pathname.endsWith("/") ? pathname : `${pathname}/`;
}

function withoutTrailingSlash(pathname: string): string {
  if (!pathname || pathname === "/") {
    return "/";
  }

  return pathname.replace(/\/+$/, "");
}

function readStoredTheme(): string | null {
  try {
    return window.localStorage.getItem(VIEWER_THEME_STORAGE_KEY);
  } catch (_) {
    return null;
  }
}

function writeStoredTheme(theme: ViewerTheme) {
  try {
    window.localStorage.setItem(VIEWER_THEME_STORAGE_KEY, theme);
  } catch (_) {}
}

function readStoredBoolean(key: string): boolean {
  try {
    return window.localStorage.getItem(key) === "true";
  } catch (_) {
    return false;
  }
}

function writeStoredBoolean(key: string, value: boolean) {
  try {
    window.localStorage.setItem(key, String(value));
  } catch (_) {}
}
