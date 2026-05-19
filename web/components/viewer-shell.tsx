"use client";

import { useEffect, useState } from "react";
import { FileList } from "./file-list";
import { ViewerChrome } from "./viewer-chrome";
import styles from "./viewer-shell.module.css";
import { FileView } from "../lib/types";
import {
  directoryRoute,
  directoryRoutePath,
  fileDirectory,
  fileRoute,
  fileRoutePath,
  firstFileIndexInDirectory,
  routePathFromPathname,
} from "../lib/presenter";

interface ViewerShellProps {
  files: FileView[];
}

export function ViewerShell({ files }: ViewerShellProps) {
  const [isOutlineOpen, setIsOutlineOpen] = useState(true);
  const [currentDirectory, setCurrentDirectory] = useState("");
  const [selectedFileIndex, setSelectedFileIndex] = useState(0);

  useEffect(() => {
    const syncSelectedFileFromPath = () => {
      const selection = resolveRouteSelection(window.location.pathname, files);
      setSelectedFileIndex(selection.fileIndex);
      setCurrentDirectory(selection.directory);
    };

    syncSelectedFileFromPath();
    window.addEventListener("popstate", syncSelectedFileFromPath);

    return () => {
      window.removeEventListener("popstate", syncSelectedFileFromPath);
    };
  }, [files]);

  const handleSelectFile = (fileIndex: number) => {
    setSelectedFileIndex(fileIndex);
    setCurrentDirectory(fileDirectory(files[fileIndex]?.path ?? ""));
    window.history.pushState(null, "", fileRoute(files[fileIndex]?.path ?? ""));
  };

  const handleNavigateDirectory = (directory: string) => {
    setCurrentDirectory(directory);

    const fileIndex = firstFileIndexInDirectory(files, directory);
    if (fileIndex === null) {
      window.history.pushState(null, "", directoryRoute(directory));
      return;
    }

    setSelectedFileIndex(fileIndex);
    window.history.pushState(null, "", fileRoute(files[fileIndex]?.path ?? ""));
  };

  return (
    <>
      <ViewerChrome
        isOutlineOpen={isOutlineOpen}
        onToggleOutline={() => setIsOutlineOpen((value) => !value)}
      />
      <main className={styles.pageShell}>
        <FileList
          currentDirectory={currentDirectory}
          files={files}
          isOutlineOpen={isOutlineOpen}
          onNavigateDirectory={handleNavigateDirectory}
          onSelectFile={handleSelectFile}
          selectedFileIndex={selectedFileIndex}
        />
      </main>
    </>
  );
}

interface RouteSelection {
  directory: string;
  fileIndex: number;
}

function resolveRouteSelection(
  pathname: string,
  files: FileView[],
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

  const directory = findDirectoryForRoutePath(routePath, files);
  const directoryFileIndex =
    directory === null ? null : firstFileIndexInDirectory(files, directory);
  if (directoryFileIndex !== null) {
    return {
      directory: directory ?? "",
      fileIndex: directoryFileIndex,
    };
  }

  return { directory: "", fileIndex: 0 };
}

function findDirectoryForRoutePath(
  routePath: string,
  files: FileView[],
): string | null {
  const directories = new Set(files.map((file) => fileDirectory(file.path)));

  for (const directory of directories) {
    if (directoryRoutePath(directory) === routePath) {
      return directory;
    }
  }

  return null;
}
