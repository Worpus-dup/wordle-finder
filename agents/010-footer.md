## Task: Page Footer

### Scope
- Add a static `<footer>` to `index.html` with an attribution line and a copyright line
- The footer appears directly after the main content (no viewport pinning; `body` stays a plain block container)
- Footer text is centered
- Author name comes from the existing `authors` field in `Cargo.toml` (no Cargo.toml change)

### Acceptance Criteria
- [x] Footer is rendered on the page as the last element of `<body>`
- [x] First line: `Designed by Eugene Turov and coded by Big Pickle™` (™ for fun, Big Pickle is not a registered trademark)
- [x] Second line: `© 2026 Eugene Turov <geny0turov@gmail.com>` (name + email match the `authors` field)
- [x] Footer sits directly after the main content (not pinned to the viewport, not `position: fixed`); `body` not turned into a flex container
- [x] Text is centered, styled with existing palette variables (no hard-coded colors)
- [x] DESIGN.md `## User Interface` gains a `### Footer` subsection describing the behavior (author/copyright placeholders generic, names not hard-coded)
- [x] No Rust changes; verification via `trunk serve` only (desktop + mobile)

### Implementation Order

**Step 1: HTML markup**
- Append `<footer>` after `</main>` and before `</body>` in `index.html`, two `<p>` lines:
  - `Designed by Eugene Turov and coded by Big Pickle™`
  - `© 2026 Eugene Turov <geny0turov@gmail.com>`

**Step 2: CSS footer styling**
- `body`: unchanged (no flex container)
- `footer`: `text-align: center;` plus palette-based, low-emphasis styling (`color: var(--color-absent)`, modest font-size and padding)

**Step 3: DESIGN.md**
- Add `### Footer` subsection to `## User Interface` (after `### Error Field`): footer appears after the content, centered, low-emphasis; attribution line mentions author + coder, copyright line uses a generic `<author>` placeholder so the doc doesn't need updating when data changes

### Known Patterns
- Palette colors via CSS variables (`:root`), no hard-coded colors
- Centered text: `text-align: center` (same as `h1`/`h2`)
- Static content: footer is pure HTML, no WASM involvement

### Scope Limits
- Do NOT change the footer text dynamically (no Rust/JS; static string)
- Do NOT use viewport pinning (`position: fixed` / sticky flex) — the footer sits directly after the main content
- Do NOT change `Cargo.toml`; the `authors` value already exists and the footer mirrors it (name + email)
- Escalate if `trunk serve` shows any layout regression to `main`

### Verification
```bash
trunk serve            # manual checks (no cargo test needed — no Rust changes):
# - Footer visible on desktop and mobile widths, directly after the main content
# - Footer text centered; first line credits Eugene Turov + Big Pickle, second line copyright with email
# - Main content layout unchanged (no compression from body flex)
```

### Implementation Details and Deviations

1. **Steps 1-2 — wording + entities** — per user correction "codded" → "coded" and no space between name and symbol (`Big Pickle™`). ™ used for fun (Big Pickle is not a registered trademark); © on line 2. Markup uses HTML entities (`&trade;`, `&copy;`, `&lt;`/`&gt;` for the email). Footer styled low-emphasis with `var(--color-absent)`, `font-size: 0.875rem`. Verified via `trunk build` (pipeline succeeds; footer present in `dist/index.html`); live visual check still pending via `trunk serve`.
2. **Step 2 revised — drop sticky-footer flex** — making `body` a flex column compressed `main`, so per user decision the body flex styling was removed entirely. The footer now appears directly after `main` (no `margin-top: auto`, no viewport pinning). Copyright line widened to include the author email (matches `Cargo.toml` `authors` value verbatim). `trunk build` re-verified.
3. **Step 3 — DESIGN.md `### Footer`** — placed after `### Error Field` as drafted and approved: generic `<author>` placeholders in both lines so the doc doesn't need updating when author data changes; only "Big Pickle™" stays literal. Applied verbatim per user approval.