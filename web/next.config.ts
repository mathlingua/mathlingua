import type { NextConfig } from "next";

const exportBasePath = normalizeBasePath(process.env.MLG_EXPORT_BASE_PATH);

/** Minimal Next.js configuration for the embedded MathLingua viewer app. */
const nextConfig: NextConfig = {
  output: process.env.MLG_STATIC_EXPORT === "1" ? "export" : undefined,
  trailingSlash: process.env.MLG_STATIC_EXPORT === "1" ? true : undefined,
  basePath: exportBasePath || undefined,
  assetPrefix: exportBasePath || undefined,
};

export default nextConfig;

function normalizeBasePath(value: string | undefined): string {
  if (!value) {
    return "";
  }

  const trimmed = value.trim();
  if (!trimmed || trimmed === "/") {
    return "";
  }

  const withSlash = trimmed.startsWith("/") ? trimmed : `/${trimmed}`;
  return withSlash.replace(/\/+$/, "");
}
