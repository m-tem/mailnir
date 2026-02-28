<p align="center">
  <img src="assets/logo.png" alt="Mailnir" width="400">
</p>

Batch email tool. Write Handlebars templates in YAML, pull data from multiple sources (CSV, JSON, YAML, TOML, or manual form input), join them together, and send via SMTP.

## How it works

A template is a `.mailnir.yml` file. Every field is a Handlebars expression. Sources are declared with join maps, and the body is rendered through Markdown and CSS inlining.

```yaml
sources:
  classes: { primary: true }
  inst:    { join: { class_id: classes.id } }
to: "{{inst.email}}"
subject: "Credentials for {{classes.name}}"
body: |
  # Hello {{inst.first_name}},
  {{#each classes.students}}
  - {{this.username}} / {{this.password}}
  {{/each}}
```

## Features

- **Multi-source data** — CSV (auto-detected separator and encoding), JSON, YAML, TOML, or form fields with auto-inferred inputs
- **Join engine** — 1:1, 1:N, composite key, and global joins across sources
- **Rendering pipeline** — three body modes: Markdown (GFM) → HTML → CSS inlining, raw HTML, or plain text. External or inline stylesheets.
- **Attachments** — templated file paths with `{{#each}}` support, validated for existence
- **Template editor** — syntax highlighting and namespace-aware autocomplete
- **Live preview** — navigate entries, HTML and plain-text tabs, per-entry validation (unresolved variables, invalid emails, missing joins, missing files)
- **SMTP sending** — multiple profiles with connection testing, OS keychain credentials, parallel sends, cancellation, retry failed entries

## Building

Requires Node.js, Rust, and platform dependencies for Tauri 2.

```sh
# Linux (Debian/Ubuntu)
sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libssl-dev

# Linux (Fedora) — or use the included distrobox.ini
sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel libxdo-devel

npm install
npm run tauri build
```

A `distrobox.ini` is included for Fedora-based development containers:

```sh
distrobox assemble create --file distrobox.ini
distrobox enter mailnir-dev
```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
