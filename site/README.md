# Site (Astro + Bun)

Static GitHub Pages for **wwm**. First doc is Getting started (`/docs/getting-started/`). `/docs/welcome/` redirects there. The product is `wwm.exe` on Windows (winget, WSL disks, profiles).

```
cd site
bun install
bun run dev
bun run build
```

Published at `https://pjmagee.github.io/wwm/` when the **GitHub Pages** workflow runs (Settings → Pages → Source: GitHub Actions). `windows/install.ps1` is copied to `/install.ps1` and `/install.txt` at build time.
