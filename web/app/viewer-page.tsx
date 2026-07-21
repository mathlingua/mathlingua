import type { Metadata } from "next";
import { connection } from "next/server";
import { ViewerShell } from "../components/viewer-shell";
import { isStaticExportBuild, loadCollectionView } from "../lib/data";
import { loadStaticExportManifest } from "../lib/static-export-build";

/**
 * The collection's own title, for the browser tab and the page `<title>`.
 *
 * The backend derives this from the `name` field of the collection's `mlg.json`,
 * so both `mlg view` and `mlg export` show the collection's name rather than a
 * generic one. Falls back to the layout's default title if the payload cannot be
 * read, since a missing title should not fail the page.
 */
export async function viewerMetadata(): Promise<Metadata> {
  try {
    const title = isStaticExportBuild()
      ? (await loadStaticExportManifest()).title
      : (await loadCollectionView()).title;
    const trimmed = title?.trim();

    return trimmed ? { title: trimmed } : {};
  } catch {
    return {};
  }
}

interface ViewerPageProps {
  /** Pathname represented by the current Next.js route. */
  initialPathname?: string;
}

/** Loads the backend view payload and renders the interactive viewer shell. */
export default async function ViewerPage({
  initialPathname = "/",
}: ViewerPageProps = {}) {
  if (isStaticExportBuild()) {
    const basePath = staticExportBasePath();
    const manifest = await loadStaticExportManifest();
    return (
      <ViewerShell
        initialManifest={manifest}
        initialPathname={initialPathname}
        routeBasePath={basePath}
        staticDataBasePath={`${basePath}/data`}
      />
    );
  }

  await connection();
  const collection = await loadCollectionView();

  return (
    <ViewerShell
      initialCollection={collection}
      initialPathname={initialPathname}
    />
  );
}

function staticExportBasePath(): string {
  const value = process.env.MLG_EXPORT_BASE_PATH?.trim();
  if (!value || value === "/") {
    return "";
  }

  const withSlash = value.startsWith("/") ? value : `/${value}`;
  return withSlash.replace(/\/+$/, "");
}
