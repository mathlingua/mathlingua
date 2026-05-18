import { ViewerShell } from "../components/viewer-shell";
import { loadCollectionView } from "../lib/data";

export default async function ViewerPage() {
  const collection = await loadCollectionView();

  return <ViewerShell files={collection.files} />;
}
