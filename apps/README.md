# apps

Custom goose apps live here. Each app is a single, self-contained HTML file
(HTML + CSS + JavaScript, no external dependencies or npm packages) that the
goose [Apps extension](../documentation/docs/mcp/apps-mcp.md) can launch in a
sandboxed, standalone window.

## Why this folder exists

By default the Apps extension stores apps in goose's data directory
(`~/.local/share/goose/apps/` on macOS/Linux). When you run goose from the root
of this repository, the extension instead uses **this** `apps/` folder, so you
can create, edit, and version your apps directly alongside the rest of the code.

Directory resolution order (see `crates/goose/src/agents/platform_extensions/apps.rs`):

1. `GOOSE_APPS_DIR` environment variable, if set.
2. An `apps/` folder in the current working directory, if it exists (this one).
3. goose's data directory (the original default).

## Creating an app

Ask goose in chat, e.g. _"Create a JSON formatter app"_. The new app is written
here as `<name>.html`. You can also drop a hand-written HTML file in this folder
and goose will pick it up.

## App format

An app is plain HTML with two optional metadata `<script>` tags goose reads:

- `application/ld+json` — JSON-LD metadata: `name`, `description`,
  `width`/`height`/`resizable`, and `mcpServers`.
- `application/x-goose-prd` — an embedded product-requirements description used
  when iterating on the app.

See `clock.html` and `chat.html` in this folder for working examples.
