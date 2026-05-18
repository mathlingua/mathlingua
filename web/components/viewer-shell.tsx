"use client";

import { useEffect, useState } from "react";
import { FileList } from "./file-list";
import { ViewerChrome } from "./viewer-chrome";
import { FileView } from "../lib/types";
import {
  fileDirectory,
  firstFileIndexInDirectory,
  makeFileAnchor,
} from "../lib/presenter";

type ViewerShellProps = {
  files: FileView[];
};

export function ViewerShell({ files }: ViewerShellProps) {
  const [isOutlineOpen, setIsOutlineOpen] = useState(true);
  const [currentDirectory, setCurrentDirectory] = useState("");
  const [selectedFileIndex, setSelectedFileIndex] = useState(0);

  useEffect(() => {
    const syncSelectedFileFromHash = () => {
      const index = parseSelectedFileIndex(window.location.hash, files.length);
      const nextIndex = index ?? 0;
      setSelectedFileIndex(nextIndex);
      setCurrentDirectory(index === null ? "" : fileDirectory(files[nextIndex]?.path ?? ""));
    };

    syncSelectedFileFromHash();
    window.addEventListener("hashchange", syncSelectedFileFromHash);

    return () => {
      window.removeEventListener("hashchange", syncSelectedFileFromHash);
    };
  }, [files]);

  const handleSelectFile = (fileIndex: number) => {
    setSelectedFileIndex(fileIndex);
    setCurrentDirectory(fileDirectory(files[fileIndex]?.path ?? ""));
    window.history.replaceState(null, "", `#${makeFileAnchor(fileIndex)}`);
  };

  const handleNavigateDirectory = (directory: string) => {
    setCurrentDirectory(directory);

    const fileIndex = firstFileIndexInDirectory(files, directory);
    if (fileIndex === null) {
      return;
    }

    setSelectedFileIndex(fileIndex);
    window.history.replaceState(null, "", `#${makeFileAnchor(fileIndex)}`);
  };

  return (
    <>
      <ViewerChrome
        isOutlineOpen={isOutlineOpen}
        onToggleOutline={() => setIsOutlineOpen((value) => !value)}
      />
      <main className="page-shell">
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

function parseSelectedFileIndex(hash: string, fileCount: number): number | null {
  const match = /^#file-(\d+)$/.exec(hash);
  if (!match) {
    return null;
  }

  const index = Number.parseInt(match[1], 10);
  if (Number.isNaN(index) || index < 0 || index >= fileCount) {
    return null;
  }

  return index;
}
