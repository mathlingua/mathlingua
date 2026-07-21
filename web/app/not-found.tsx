import { viewerMetadata } from "./viewer-page";

/** Titles the not-found page with the collection's own name. */
export const generateMetadata = viewerMetadata;

/** Fallback page shown for routes that are not part of the collection. */
export default function NotFound() {
  return (
    <main>
      <h1>Page not found</h1>
      <p>This page is not part of the collection.</p>
    </main>
  );
}
