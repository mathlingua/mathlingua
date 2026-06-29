import { ViewerShell } from "../components/viewer-shell";
import { loadCollectionView } from "../lib/data";

interface ViewerPageProps {
  /** Pathname represented by the current Next.js route. */
  initialPathname?: string;
}

/** Loads the backend view payload and renders the interactive viewer shell. */
export default async function ViewerPage({
  initialPathname = "/",
}: ViewerPageProps = {}) {
  const collection = await loadCollectionView();

  return (
    <ViewerShell
      directories={collection.directories}
      files={collection.files}
      initialPathname={initialPathname}
    />
  );
}
