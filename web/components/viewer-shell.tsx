"use client";

import { useEffect, useState } from "react";
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
import { DirectoryView, FileView } from "../lib/types";
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
  /** Renderable directories for the current collection. */
  directories: DirectoryView[];
  /** Renderable files for the current collection. */
  files: FileView[];
  /** Initial browser pathname supplied by the server route. */
  initialPathname: string;
}

/** Owns browser history, selected file, outline directory, and chrome state. */
export function ViewerShell({
  directories,
  files,
  initialPathname,
}: ViewerShellProps) {
  const initialSelection = resolveRouteSelection(
    initialPathname,
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
        window.location.pathname,
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
  }, [files, directories]);

  const handleSelectFile = (fileIndex: number) => {
    setSelectedFileIndex(fileIndex);
    setCurrentDirectory(fileDirectory(files[fileIndex]?.path ?? ""));
    window.history.pushState(null, "", fileRoute(files[fileIndex]?.path ?? ""));
  };

  const handleNavigateDirectory = (directory: string) => {
    setCurrentDirectory(directory);

    const fileIndex = firstFileIndexInDirectory(files, directories, directory);
    if (fileIndex === null) {
      window.history.pushState(null, "", directoryRoute(directory));
      return;
    }

    setSelectedFileIndex(fileIndex);
    window.history.pushState(null, "", fileRoute(files[fileIndex]?.path ?? ""));
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
          directories={directories}
          files={files}
          onCloseOutline={() => setOutlineState("closed")}
          onNavigateDirectory={handleNavigateDirectory}
          onSelectFile={handleSelectFile}
          outlineState={outlineState}
          selectedFileIndex={selectedFileIndex}
        />
      </main>
    </>
  );
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
