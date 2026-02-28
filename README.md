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

- **Multi-source data** — CSV (auto-detected separator/encoding), JSON, YAML, TOML, or inline form fields
- **Join engine** — 1:1, 1:N, composite key, and global joins across sources
- **Rendering pipeline** — Handlebars → Markdown (GFM) → HTML → CSS inlining
- **Live preview** — navigate entries, see rendered HTML/text, per-entry validation warnings
- **Template editor** — syntax highlighting and namespace-aware autocomplete
- **SMTP sending** — multiple profiles, OS keychain credentials, parallel sends, retry failed entries

## Building

Requires Node.js, Rust, and platform dependencies for Tauri 2.

```sh
# Linux (Debian/Ubuntu)
sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libssl-dev

npm install
npm run tauri build
```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
