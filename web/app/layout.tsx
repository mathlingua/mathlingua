import type { Metadata } from "next";
import "katex/dist/katex.min.css";
import "./globals.css";

/** Metadata shown by browsers and Next.js for the local collection viewer. */
export const metadata: Metadata = {
  title: "Mathlingua Viewer",
  description: "Rendered Mathlingua collection viewer",
};

/** Root document shell shared by all viewer routes. */
export default function RootLayout({
  children,
}: Readonly<{
  /** Route contents rendered by Next.js. */
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
