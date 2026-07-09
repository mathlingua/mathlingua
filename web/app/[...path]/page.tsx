import ViewerPage from "../viewer-page";
import { staticExportRouteParams } from "../../lib/static-export-build";

interface PathViewerPageProps {
  /** Catch-all route params supplied by Next.js. */
  params: Promise<{
    /** URL path segments after the root route. */
    path?: string[];
  }>;
}

/** Enumerates static deep-link pages for `mlg export`. */
export async function generateStaticParams() {
  return staticExportRouteParams();
}

/** Catch-all viewer route for file and directory paths. */
export default async function PathViewerPage({ params }: PathViewerPageProps) {
  const { path = [] } = await params;

  return <ViewerPage initialPathname={`/${path.join("/")}`} />;
}
