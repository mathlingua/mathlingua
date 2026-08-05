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
