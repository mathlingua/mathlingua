import { ViewerShell } from "../components/viewer-shell";
import { loadCollectionView } from "../lib/data";

/** Loads the backend view payload and renders the interactive viewer shell. */
export default async function ViewerPage() {
  const collection = await loadCollectionView();

  return (
    <ViewerShell
      directories={collection.directories}
      files={collection.files}
    />
  );
}
