import { connection } from "next/server";
import { ViewerShell } from "../components/viewer-shell";
import { isStaticExportBuild, loadCollectionView } from "../lib/data";
import { loadStaticExportManifest } from "../lib/static-export-build";

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
