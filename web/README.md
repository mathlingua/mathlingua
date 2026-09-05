# MathLingua Viewer

This directory contains the React/Vite frontend used by `mlg view` and
`mlg export`.

The production build in `dist/` is committed because Cargo embeds it into the
`mlg` executable. After changing frontend source, refresh those assets with:

```sh
npm install
npm run build
```

Node.js and npm are required only for this contributor workflow. Users running
an installed `mlg` executable do not need either tool.

The outline header shows the directory currently being browsed. Use **Up** or
an entry's right arrow to browse directories without changing the open page or
browser history. Click a file or directory name (including the header) to open
its page. Opening a page, using Previous/Next, or using browser Back/Forward
brings the outline to that page's directory. On narrow screens, browsing keeps
the outline open; selecting a page closes it.
