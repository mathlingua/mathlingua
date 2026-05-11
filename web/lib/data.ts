import fs from "node:fs/promises";
import { CollectionView } from "./types";

export async function loadCollectionView(): Promise<CollectionView> {
  const dataPath = process.env.MLG_VIEW_DATA_PATH;

  if (!dataPath) {
    throw new Error("MLG_VIEW_DATA_PATH is not set");
  }

  const json = await fs.readFile(dataPath, "utf8");
  return JSON.parse(json) as CollectionView;
}
