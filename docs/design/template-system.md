# Template System

## File Format

Templates are YAML files (`.mailnir.yml`). Every value is a Handlebars template string. Data bindings are a top-level key.

```yaml
sources:
  classes: { primary: true }
  inst:    { join: { class_id: classes.id } }
  contacts: { join: { name: inst.name } }

to: "{{contacts.email}}"
cc: ""
bcc: ""
subject: "Credentials for {{classes.name}}"

body: |
  # Hello {{inst.first_name}},

  Here are the credentials for **{{classes.name}}**:
  {{#each classes.students}}
  - {{this.username}} / {{this.password}}
  {{/each}}

attachments: |
  {{#each classes.students}}
  reports/{{this.username}}.pdf
  {{/each}}
```

Benefits: robust parsing via any YAML library, YAML anchors (`&`/`*`) for reusable fragments, no ambiguity between fields, `|` block scalars for multiline body/attachments. Joins are YAML maps — no expression parser needed. Composite joins use multiple keys:

```yaml
inst:
  join:
    class_id: classes.id
    semester: classes.semester
```

## Attachments

The `attachments` field renders to one file path per line. `{{#each}}` handles variable-length lists naturally.

## Stylesheets

Optional CSS for HTML emails. Email clients strip `<style>` tags, so `css-inline` converts rules to inline `style` attributes at render time.

Two options (mutually exclusive):
- **`stylesheet`**: Path to a `.css` file. `stylesheet: styles/email.css`
- **`style`**: Inline CSS in the YAML. `style: | h1 { color: #333; }`

## Body Rendering Modes

- **Markdown** (default): Handlebars → Markdown → HTML → CSS inline → final HTML email + plain-text fallback.
- **Raw HTML**: Opt-in via `body_format: html` — Handlebars → HTML → CSS inline. No markdown step.
- **Plain text only**: Opt-in via `body_format: text` — Handlebars → plain text. No HTML part, stylesheet ignored.
