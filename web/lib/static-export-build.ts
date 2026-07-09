import fs from "node:fs/promises";
import path from "node:path";
import type { CollectionManifest } from "./types";

/** Reads the export manifest while Next statically enumerates routes. */
export async function staticExportRouteParams(): Promise<{ path: string[] }[]> {
  if (process.env.MLG_STATIC_EXPORT !== "1") {
    return [];
  }

  const manifest = await loadStaticExportManifest();
  const routes = new Set<string>();

  for (const file of manifest.files) {
    const route = fileRoutePath(file.path);
    if (route) {
      routes.add(route);
    }
  }

  for (const directory of manifest.directories) {
    const route = directoryRoutePath(directory.path);
    if (route) {
      routes.add(route);
    }
  }

  return [...routes].sort().map((route) => ({
    path: route.split("/").filter(Boolean),
  }));
}

/** Reads the small static export manifest during Next static rendering. */
export async function loadStaticExportManifest(): Promise<CollectionManifest> {
  const dataDir = process.env.MLG_EXPORT_DATA_DIR;
  if (!dataDir) {
    throw new Error("MLG_EXPORT_DATA_DIR is not set");
  }

  const json = await fs.readFile(path.join(dataDir, "manifest.json"), "utf8");
  return JSON.parse(json) as CollectionManifest;
}

function fileRoutePath(filePath: string): string {
  return normalizeRoutePath(
    contentRelativePath(filePath).replace(/\.mlg$/i, ""),
  );
}

function directoryRoutePath(directoryPath: string): string {
  return normalizeRoutePath(contentRelativePath(directoryPath));
}

function contentRelativePath(value: string): string {
  const normalized = value.replace(/\\/g, "/").replace(/^\/+/, "");
  return normalized.startsWith("content/")
    ? normalized.slice("content/".length)
    : normalized;
}

function normalizeRoutePath(value: string): string {
  return value
    .trim()
    .replace(/\s+/g, "_")
    .replace(/\/+/g, "/")
    .replace(/^\/+|\/+$/g, "");
}
