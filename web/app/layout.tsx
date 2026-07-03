import type { Metadata } from "next";
import "katex/dist/katex.min.css";
import "./globals.css";

/** Metadata shown by browsers and Next.js for the local collection viewer. */
export const metadata: Metadata = {
  title: "Mathlingua Viewer",
  description: "Rendered Mathlingua collection viewer",
};

const THEME_BOOTSTRAP_SCRIPT = `
(() => {
  try {
    const theme = window.localStorage.getItem("mlg-view-theme");
    if (!/^(classic|mono|sepia|retro|dark)$/.test(theme ?? "")) {
      return;
    }
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme === "dark" ? "dark" : "light";
  } catch (_) {}
})();
`;

/** Root document shell shared by all viewer routes. */
export default function RootLayout({
  children,
}: Readonly<{
  /** Route contents rendered by Next.js. */
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_BOOTSTRAP_SCRIPT }} />
      </head>
      <body>{children}</body>
    </html>
  );
}
