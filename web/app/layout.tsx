import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Mathlingua Viewer",
  description: "Rendered Mathlingua collection viewer",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
