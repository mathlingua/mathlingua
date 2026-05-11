import { ViewerShell } from "../components/viewer-shell";
import { loadCollectionView } from "../lib/data";

export const dynamic = "force-dynamic";

export default async function HomePage() {
  const collection = await loadCollectionView();

  return <ViewerShell files={collection.files} />;
}
