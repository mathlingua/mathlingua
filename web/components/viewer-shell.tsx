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
  directoryRoute,
  directoryRoutePath,
  fileDirectory,
  fileRoute,
  fileRoutePath,
  firstFileIndexInDirectory,
  routePathFromPathname,
} from "../lib/presenter";

const NARROW_OUTLINE_MEDIA_QUERY = "(max-width: 860px)";

/** Props for the client-side viewer state container. */
interface ViewerShellProps {
  /** Full collection view supplied by `mlg view` development mode. */
  initialCollection?: CollectionView;
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
  const initialSelection = resolveRouteSelection(
    stripRouteBasePath(initialPathname, routeBasePath),
    files,
    directories,
  );
  const [outlineState, setOutlineState] = useState<OutlineState>("auto");
  const [currentDirectory, setCurrentDirectory] = useState(
    initialSelection.directory,
  );
  const [selectedFileIndex, setSelectedFileIndex] = useState(
    initialSelection.fileIndex,
  );
  const [theme, setTheme] = useState<ViewerTheme>(DEFAULT_VIEWER_THEME);

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
  }, []);

  useEffect(() => {
    const syncSelectedFileFromPath = () => {
      const selection = resolveRouteSelection(
        stripRouteBasePath(window.location.pathname, routeBasePath),
        files,
        directories,
      );
      setSelectedFileIndex(selection.fileIndex);
      setCurrentDirectory(selection.directory);
    };

    syncSelectedFileFromPath();
    window.addEventListener("popstate", syncSelectedFileFromPath);

    return () => {
      window.removeEventListener("popstate", syncSelectedFileFromPath);
    };
  }, [files, directories, routeBasePath]);

  const handleSelectFile = (fileIndex: number) => {
    setSelectedFileIndex(fileIndex);
    setCurrentDirectory(fileDirectory(files[fileIndex]?.path ?? ""));
    window.history.pushState(
      null,
      "",
      withRouteBasePath(
        fileRoute(files[fileIndex]?.path ?? ""),
        routeBasePath,
        useTrailingSlashRoutes,
      ),
    );
  };

  const handleNavigateDirectory = (directory: string) => {
    setCurrentDirectory(directory);

    const fileIndex = firstFileIndexInDirectory(files, directories, directory);
    if (fileIndex === null) {
      window.history.pushState(
        null,
        "",
        withRouteBasePath(
          directoryRoute(directory),
          routeBasePath,
          useTrailingSlashRoutes,
        ),
      );
      return;
    }

    setSelectedFileIndex(fileIndex);
    window.history.pushState(
      null,
      "",
      withRouteBasePath(
        fileRoute(files[fileIndex]?.path ?? ""),
        routeBasePath,
        useTrailingSlashRoutes,
      ),
    );
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
        onThemeChange={handleThemeChange}
        outlineState={outlineState}
        theme={theme}
      />
      <main className={styles.pageShell}>
        <FileList
          currentDirectory={currentDirectory}
          definitionItemIds={manifest?.definitions}
          directories={directories}
          files={files}
          loadError={selectedFileLoadError}
          isSelectedFileLoading={isSelectedFileLoading}
          loadedDefinitions={loadedDefinitions}
          onCloseOutline={() => setOutlineState("closed")}
          onLoadDefinition={handleLoadDefinition}
          onNavigateDirectory={handleNavigateDirectory}
          onSelectFile={handleSelectFile}
          outlineState={outlineState}
          selectedFileIndex={selectedFileIndex}
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

/** Route-derived viewer selection state. */
interface RouteSelection {
  /** Directory shown in the outline. */
  directory: string;
  /** File index shown in the document stream. */
  fileIndex: number;
}

/** Resolves a browser pathname into the selected file and outline directory. */
function resolveRouteSelection(
  pathname: string,
  files: FileView[],
  directories: DirectoryView[],
): RouteSelection {
  const routePath = routePathFromPathname(pathname);
  if (!routePath) {
    return { directory: "", fileIndex: 0 };
  }

  const fileIndex = files.findIndex(
    (file) => fileRoutePath(file.path) === routePath,
  );
  if (fileIndex >= 0) {
    return {
      directory: fileDirectory(files[fileIndex]?.path ?? ""),
      fileIndex,
    };
  }

  const directory = findDirectoryForRoutePath(routePath, files, directories);
  const directoryFileIndex =
    directory === null
      ? null
      : firstFileIndexInDirectory(files, directories, directory);
  if (directoryFileIndex !== null) {
    return {
      directory: directory ?? "",
      fileIndex: directoryFileIndex,
    };
  }

  return { directory: "", fileIndex: 0 };
}

/** Finds the outline directory represented by a normalized route path. */
function findDirectoryForRoutePath(
  routePath: string,
  files: FileView[],
  directories: DirectoryView[],
): string | null {
  const directoryPaths = new Set([
    ...files.map((file) => fileDirectory(file.path)),
    ...directories.map((directory) => fileDirectory(`${directory.path}/_`)),
  ]);

  for (const directory of directoryPaths) {
    if (directoryRoutePath(directory) === routePath) {
      return directory;
    }
  }

  return null;
}
