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
      const index = parseSelectedFileIndex(window.location.hash, files);
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
    window.history.replaceState(
      null,
      "",
      `#${makeFileAnchor(files[fileIndex]?.path ?? "")}`,
    );
  };

  const handleNavigateDirectory = (directory: string) => {
    setCurrentDirectory(directory);

    const fileIndex = firstFileIndexInDirectory(files, directory);
    if (fileIndex === null) {
      return;
    }

    setSelectedFileIndex(fileIndex);
    window.history.replaceState(
      null,
      "",
      `#${makeFileAnchor(files[fileIndex]?.path ?? "")}`,
    );
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

function parseSelectedFileIndex(hash: string, files: FileView[]): number | null {
  const rawAnchor = hash.replace(/^#/, "");
  if (!rawAnchor) {
    return null;
  }

  const legacyMatch = /^file-(\d+)$/.exec(rawAnchor);
  if (legacyMatch) {
    const legacyIndex = Number.parseInt(legacyMatch[1], 10);
    return Number.isNaN(legacyIndex) ||
      legacyIndex < 0 ||
      legacyIndex >= files.length
      ? null
      : legacyIndex;
  }

  const anchor = decodeURIComponent(rawAnchor);
  const index = files.findIndex(
    (file) => decodeURIComponent(makeFileAnchor(file.path)) === anchor,
  );

  return index < 0 ? null : index;
}
