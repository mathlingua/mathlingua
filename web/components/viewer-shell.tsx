"use client";

import { useState } from "react";
import { FileList } from "./file-list";
import { ViewerChrome } from "./viewer-chrome";
import { FileView } from "../lib/types";

type ViewerShellProps = {
  files: FileView[];
};

export function ViewerShell({ files }: ViewerShellProps) {
  const [isOutlineOpen, setIsOutlineOpen] = useState(true);

  return (
    <>
      <ViewerChrome
        isOutlineOpen={isOutlineOpen}
        onToggleOutline={() => setIsOutlineOpen((value) => !value)}
      />
      <main className="page-shell">
        <FileList files={files} isOutlineOpen={isOutlineOpen} />
      </main>
    </>
  );
}
