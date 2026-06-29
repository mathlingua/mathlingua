import ViewerPage from "../viewer-page";

/** Ensures every deep viewer route reads the current CLI-generated payload. */
export const dynamic = "force-dynamic";

interface PathViewerPageProps {
  /** Catch-all route params supplied by Next.js. */
  params: Promise<{
    /** URL path segments after the root route. */
    path?: string[];
  }>;
}

/** Catch-all viewer route for file and directory paths. */
export default async function PathViewerPage({ params }: PathViewerPageProps) {
  const { path = [] } = await params;

  return <ViewerPage initialPathname={`/${path.join("/")}`} />;
}
