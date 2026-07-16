# Generated site project

This project packages a generated HTML/CSS site and exposes a separate content check.

Run from this directory:

```bash
mantle build
mantle build .#site
mantle build .#checks.site-content
```

The package contains `index.html`, `assets/site.css`, and `manifest.txt`. The check verifies that the generated files agree and writes `result.txt`.
