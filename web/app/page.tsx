import ViewerPage, { viewerMetadata } from "./viewer-page";

/** Titles the root route with the collection's own name. */
export const generateMetadata = viewerMetadata;

/** Root viewer route. */
export default function RootViewerPage() {
  return <ViewerPage initialPathname="/" />;
}
