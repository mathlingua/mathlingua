import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";

const BASE_HREF_MARKER = "__MLG_BASE_HREF__";
const VIEW_CONFIG_MARKER = "__MLG_RUNTIME_CONFIG_JSON__";

/** Development-only bridge to the collection JSON emitted by the Rust CLI. */
function mathlinguaDevelopmentData(): Plugin {
  return {
    name: "mathlingua-development-data",
    transformIndexHtml(html, context) {
      if (!context.server) {
        return html;
      }

      return html
        .replace(BASE_HREF_MARKER, "/")
        .replace(
          VIEW_CONFIG_MARKER,
          JSON.stringify({ collectionDataPath: "/api/collection.json" }),
        );
    },
    configureServer(server) {
      server.middlewares.use(
        "/api/collection.json",
        (_request: IncomingMessage, response: ServerResponse, next) => {
          const dataPath = process.env.MLG_VIEW_DATA_PATH;
          if (!dataPath) {
            next();
            return;
          }

          response.setHeader("Content-Type", "application/json; charset=utf-8");
          response.setHeader("Cache-Control", "no-store");
          fs.createReadStream(dataPath).pipe(response);
        },
      );
    },
  };
}

export default defineConfig({
  base: "./",
  plugins: [react(), mathlinguaDevelopmentData()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
