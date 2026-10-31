"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { FileList } from "./file-list";
import type { OutlineState } from "./outline-state";
import { ViewerChrome } from "./viewer-chrome";
import {
  DEFAULT_VIEWER_FONT,
  DEFAULT_VIEWER_FONT_SCALE,
  VIEWER_FONT_SCALE_STORAGE_KEY,
  VIEWER_FONT_STORAGE_KEY,
  applyViewerFont,
  applyViewerFontScale,
  isViewerFont,
  parseViewerFontScale,
  stepViewerFontScale,
  type ViewerFont,
  type ViewerFontScale,
} from "./viewer-font";
import {
  DEFAULT_VIEWER_THEME,
  DEFAULT_VIEWER_THEME_FAMILY,
  DEFAULT_VIEWER_THEME_MODE,
  VIEWER_THEME_FAMILY_STORAGE_KEY,
  VIEWER_THEME_MODE_STORAGE_KEY,
  VIEWER_THEME_STORAGE_KEY,
  applyViewerTheme,
  getFamilyForTheme,
  isViewerTheme,
  isViewerThemeFamily,
  isViewerThemeMode,
  resolveViewerTheme,
  type ViewerTheme,
  type ViewerThemeMode,
} from "./viewer-theme";
import {
  DEFAULT_VIEWER_SPACING,
  VIEWER_SPACING_STORAGE_KEY,
  applyViewerSpacing,
  isViewerSpacing,
  type ViewerSpacing,
} from "./viewer-spacing";
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
  const [collectionLoadError, setCollectionLoadError] = useState<string | null>(
    null,
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
  const [themeFamily, setThemeFamily] = useState<string>(
    DEFAULT_VIEWER_THEME_FAMILY,
  );
  const [themeMode, setThemeMode] = useState<ViewerThemeMode>(
    DEFAULT_VIEWER_THEME_MODE,
  );
  const [theme, setTheme] = useState<ViewerTheme>(DEFAULT_VIEWER_THEME);
  const [font, setFont] = useState<ViewerFont>(DEFAULT_VIEWER_FONT);
  const [spacing, setSpacing] = useState<ViewerSpacing>(DEFAULT_VIEWER_SPACING);
  const [fontScale, setFontScale] = useState<ViewerFontScale>(
    DEFAULT_VIEWER_FONT_SCALE,
  );
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
        if (!cancelled) {
          setCollectionLoadError(readableErrorMessage(error));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [collectionDataPath, manifest]);

  useEffect(() => {
    if (!collectionDataPath) {
      return;
    }

    let cancelled = false;
    let lastVersion: number | null = null;
    const versionUrl = collectionDataPath.replace(
      /\/api\/collection\.json$/,
      "/api/version",
    );

    const intervalId = window.setInterval(() => {
      fetchJson<{ version: number }>(versionUrl)
        .then((data) => {
          if (cancelled) {
            return;
          }
          if (lastVersion === null) {
            lastVersion = data.version;
            return;
          }
          if (data.version !== lastVersion) {
            lastVersion = data.version;
            return fetchJson<CollectionView>(collectionDataPath).then(
              (collection) => {
                if (cancelled) {
                  return;
                }
                setManifest(manifestFromCollection(collection));
                setLoadedFiles(
                  Object.fromEntries(
                    collection.files.map((file) => [file.path, file]),
                  ),
                );
              },
            );
          }
        })
        .catch(() => {
          // Ignore polling errors while rebuilding/restarting
        });
    }, 500);

    return () => {
      cancelled = true;
      window.clearInterval(intervalId);
    };
  }, [collectionDataPath]);

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
        if (!cancelled) {
          setCollectionLoadError(readableErrorMessage(error));
        }
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
    const storedFamily = readStoredThemeFamily();
    const storedMode = readStoredThemeMode();
    const documentTheme = document.documentElement.dataset.theme;
    const storedTheme = readStoredTheme();

    let initialFamily: string;
    let initialMode: ViewerThemeMode;

    if (storedFamily && isViewerThemeFamily(storedFamily)) {
      initialFamily = storedFamily;
      initialMode =
        storedMode && isViewerThemeMode(storedMode)
          ? storedMode
          : DEFAULT_VIEWER_THEME_MODE;
    } else {
      const candidateTheme = isViewerTheme(documentTheme)
        ? documentTheme
        : isViewerTheme(storedTheme)
          ? storedTheme
          : DEFAULT_VIEWER_THEME;
      const parsed = getFamilyForTheme(candidateTheme);
      initialFamily = parsed.family.id;
      initialMode = parsed.mode;
    }

    const initialTheme = resolveViewerTheme(initialFamily, initialMode);
    setThemeFamily(initialFamily);
    setThemeMode(initialMode);
    setTheme(initialTheme);
    applyViewerTheme(initialTheme);
    writeStoredThemeFamily(initialFamily);
    writeStoredThemeMode(initialMode);
    writeStoredTheme(initialTheme);

    const documentFont = document.documentElement.dataset.font;
    const storedFont = readStoredFont();
    const initialFont = isViewerFont(documentFont)
      ? documentFont
      : isViewerFont(storedFont)
        ? storedFont
        : DEFAULT_VIEWER_FONT;

    const documentFontScale = document.documentElement.style.getPropertyValue(
      "--viewer-font-scale",
    );
    const storedFontScale = readStoredFontScale();
    const initialFontScale =
      parseViewerFontScale(documentFontScale) ??
      parseViewerFontScale(storedFontScale) ??
      DEFAULT_VIEWER_FONT_SCALE;

    const storedSpacing = readStoredSpacing();
    const initialSpacing = isViewerSpacing(storedSpacing)
      ? storedSpacing
      : DEFAULT_VIEWER_SPACING;

    setFont(initialFont);
    applyViewerFont(initialFont);
    setSpacing(initialSpacing);
    applyViewerSpacing(initialSpacing);
    setFontScale(initialFontScale);
    applyViewerFontScale(initialFontScale);
    setShowTypes(readStoredBoolean(SHOW_TYPES_STORAGE_KEY));
  }, []);

  useEffect(() => {
    if (themeMode !== "auto") {
      return;
    }
    const mediaQuery = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (!mediaQuery) {
      return;
    }
    const handleChange = () => {
      const resolved = resolveViewerTheme(themeFamily, "auto");
      setTheme(resolved);
      applyViewerTheme(resolved);
      writeStoredTheme(resolved);
    };
    mediaQuery.addEventListener?.("change", handleChange);
    return () => {
      mediaQuery.removeEventListener?.("change", handleChange);
    };
  }, [themeFamily, themeMode]);

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

  const handleThemeFamilyChange = (nextFamily: string) => {
    setThemeFamily(nextFamily);
    writeStoredThemeFamily(nextFamily);
    const resolved = resolveViewerTheme(nextFamily, themeMode);
    setTheme(resolved);
    applyViewerTheme(resolved);
    writeStoredTheme(resolved);
  };

  const handleThemeModeChange = (nextMode: ViewerThemeMode) => {
    setThemeMode(nextMode);
    writeStoredThemeMode(nextMode);
    const resolved = resolveViewerTheme(themeFamily, nextMode);
    setTheme(resolved);
    applyViewerTheme(resolved);
    writeStoredTheme(resolved);
  };

  const handleFontChange = (nextFont: ViewerFont) => {
    setFont(nextFont);
    applyViewerFont(nextFont);
    writeStoredFont(nextFont);
  };

  const handleSpacingChange = (nextSpacing: ViewerSpacing) => {
    setSpacing(nextSpacing);
    applyViewerSpacing(nextSpacing);
    writeStoredSpacing(nextSpacing);
  };

  const handleDecreaseFontScale = () => {
    setFontScale((current) => {
      const next = stepViewerFontScale(current, -1);
      applyViewerFontScale(next);
      writeStoredFontScale(next);
      return next;
    });
  };

  const handleIncreaseFontScale = () => {
    setFontScale((current) => {
      const next = stepViewerFontScale(current, 1);
      applyViewerFontScale(next);
      writeStoredFontScale(next);
      return next;
    });
  };

  const handleResetFontScale = () => {
    setFontScale(DEFAULT_VIEWER_FONT_SCALE);
    applyViewerFontScale(DEFAULT_VIEWER_FONT_SCALE);
    writeStoredFontScale(DEFAULT_VIEWER_FONT_SCALE);
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
  const isCollectionLoading = Boolean(
    (collectionDataPath || staticDataBasePath) &&
    !manifest &&
    !collectionLoadError,
  );
  const isSelectedFileLoading = Boolean(
    staticDataBasePath &&
    selectedFilePath &&
    !selectedFileLoadError &&
    (loadingFilePaths[selectedFilePath] || !loadedFiles[selectedFilePath]),
  );

  return (
    <>
      <ViewerChrome
        font={font}
        fontScale={fontScale}
        onDecreaseFontScale={handleDecreaseFontScale}
        onFontChange={handleFontChange}
        onIncreaseFontScale={handleIncreaseFontScale}
        onResetFontScale={handleResetFontScale}
        onSpacingChange={handleSpacingChange}
        onThemeFamilyChange={handleThemeFamilyChange}
        onThemeModeChange={handleThemeModeChange}
        onToggleOutline={handleToggleOutline}
        onToggleTypes={handleToggleTypes}
        outlineState={outlineState}
        showTypes={showTypes}
        spacing={spacing}
        theme={theme}
        themeFamily={themeFamily}
        themeMode={themeMode}
      />
      <main className={styles.pageShell}>
        <FileList
          collectionPreface={manifest?.preface}
          collectionTitle={manifest?.title}
          currentDirectory={currentDirectory}
          definitionItemIds={manifest?.definitions}
          directories={directories}
          files={files}
          isCollectionLoading={isCollectionLoading}
          loadError={collectionLoadError ?? selectedFileLoadError}
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

function readStoredThemeFamily(): string | null {
  try {
    return window.localStorage.getItem(VIEWER_THEME_FAMILY_STORAGE_KEY);
  } catch (_) {
    return null;
  }
}

function writeStoredThemeFamily(familyId: string) {
  try {
    window.localStorage.setItem(VIEWER_THEME_FAMILY_STORAGE_KEY, familyId);
  } catch (_) {}
}

function readStoredThemeMode(): ViewerThemeMode | null {
  try {
    const val = window.localStorage.getItem(VIEWER_THEME_MODE_STORAGE_KEY);
    return isViewerThemeMode(val) ? val : null;
  } catch (_) {
    return null;
  }
}

function writeStoredThemeMode(mode: ViewerThemeMode) {
  try {
    window.localStorage.setItem(VIEWER_THEME_MODE_STORAGE_KEY, mode);
  } catch (_) {}
}

function readStoredFont(): string | null {
  try {
    return window.localStorage.getItem(VIEWER_FONT_STORAGE_KEY);
  } catch (_) {
    return null;
  }
}

function writeStoredFont(font: ViewerFont) {
  try {
    window.localStorage.setItem(VIEWER_FONT_STORAGE_KEY, font);
  } catch (_) {}
}

function readStoredSpacing(): string | null {
  try {
    return window.localStorage.getItem(VIEWER_SPACING_STORAGE_KEY);
  } catch (_) {
    return null;
  }
}

function writeStoredSpacing(spacing: ViewerSpacing) {
  try {
    window.localStorage.setItem(VIEWER_SPACING_STORAGE_KEY, spacing);
  } catch (_) {}
}

function readStoredFontScale(): string | null {
  try {
    return window.localStorage.getItem(VIEWER_FONT_SCALE_STORAGE_KEY);
  } catch (_) {
    return null;
  }
}

function writeStoredFontScale(scale: ViewerFontScale) {
  try {
    window.localStorage.setItem(VIEWER_FONT_SCALE_STORAGE_KEY, String(scale));
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
