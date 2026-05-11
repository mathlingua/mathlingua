import { FileList } from "../components/file-list";
import { ViewerChrome } from "../components/viewer-chrome";
import { loadCollectionView } from "../lib/data";

export const dynamic = "force-dynamic";

export default async function HomePage() {
  const collection = await loadCollectionView();

  return (
    <>
      <ViewerChrome title={collection.title} />
      <main className="page-shell">
        <FileList files={collection.files} />
      </main>
    </>
  );
}
