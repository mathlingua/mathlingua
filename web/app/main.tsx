import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "katex/dist/katex.min.css";
import "./globals.css";
import { ViewerShell } from "../components/viewer-shell";

export type ViewerRuntimeConfig = {
  routeBasePath?: string;
  collectionDataPath?: string;
  staticDataBasePath?: string;
};

declare global {
  interface Window {
    __MLG_VIEW_CONFIG__?: ViewerRuntimeConfig;
  }
}

const config = window.__MLG_VIEW_CONFIG__ ?? {};
const root = document.getElementById("root");

if (!root) {
  throw new Error("MathLingua viewer root element is missing");
}

createRoot(root).render(
  <StrictMode>
    <ViewerShell
      collectionDataPath={config.collectionDataPath}
      initialPathname={window.location.pathname}
      routeBasePath={config.routeBasePath}
      staticDataBasePath={config.staticDataBasePath}
    />
  </StrictMode>,
);
