## Task: GitHub Actions — CI/CD Build & Deploy to GitHub Pages

<!-- Human-authored, post-factum. "H" = human/hand: created manually outside the
     standard sanctioned task workflow, but captured here as reference for
     future modifications. -->

### Status
- **Status**: COMPLETED (post-factum). File: `.github/workflows/publish-page.yaml`.
- **Goal**: Create CI/CD for testing and deployment of the Wordle Finder app,
  building the trunk WASM bundle and publishing it to GitHub Pages.

### Motivation
- Deliver the app as a live, publicly accessible page via GitHub Pages.
- Run tests automatically in CI before deployment (test → build → deploy).
- Provide a canonical reference for future CI/CD modifications.

### Workflow Overview (`.github/workflows/publish-page.yaml`)
- **Trigger**: `on: [push]`
- **Jobs**:
  1. **`test`** — every push: `actions/checkout@v6`, `cargo test`.
  2. **`build`** — only `main`: checkout, add `wasm32-unknown-unknown`,
     install trunk (`jetli/trunk-action@v0.5.1`), `trunk build --release
     --public-url="./"`, upload `dist/` via `actions/upload-pages-artifact@v5`.
  3. **`deploy`** — only `main`, `needs: [test, build]`,
     `permissions: {contents: read, pages: write, id-token: write}`,
     `actions/deploy-pages@v4`.

### Key Decisions & Why
- **Relative base path `--public-url="./"`**: GitHub Pages serves project sites
  from a subpath (`https://<user>.github.io/<repo>/`). Root-absolute asset URLs
  (default `/...`) break there; `./` keeps asset paths relative so they resolve
  under any served path. Works for both project and user sites.
- **Test / build / deploy split**: tests run on every push (fast CI feedback);
  build+deploy restricted to `main` (production branch) via
  `github.ref == 'refs/heads/main'`.
- **`actions/deploy-pages` official flow** (not `gh-pages` branch): no branch
  pollution; Pages "Source: GitHub Actions" must be enabled in repo settings.

### Pitfalls Fixed / Lessons Learned (from initial version)
1. **`--public-url` cannot use a deploy-time value.** Original used
   `--public-url="${{ steps.deployment.outputs.page_url }}"` — but that output
   belongs to the `deploy` job and isn't available when `trunk build` runs in the
   `build` job (empty → trunk falls back to `/`, broken asset paths). Use a
   static relative path instead.
2. **Required `permissions`.** Pages deployment needs
   `pages: write` + `id-token: write` (plus `contents: read`). A wrong value
   (e.g. `packages: write`) causes the deploy to be denied. `deploy-pages` and
   `upload-pages-artifact` both require these.
3. **Trunk install method.** `cargo binstall trunk` fails on `ubuntu-latest`
   (cargo-binstall not preinstalled). Use `jetli/trunk-action` (or
   `cargo install trunk`).
4. **Guard deploy, not just steps.** The deploy job and its artifact should be
   gated on `main` to avoid wasteful/empty runs on feature-branch pushes.

### Future Improvements (not yet implemented)
- **Cargo dependency caching**: add `actions/cache@v4` to cache Cargo
  registries/build artifacts (`~/.cargo`, `target/`) so `cargo test` and the
  trunk build are faster on repeated runs. Not yet added.
- Add `concurrency` (cancel in-flight runs of the same ref) to avoid deploy races.
- Add `workflow_dispatch` for manual triggering.

### Requirements / Prerequisites
- Repo must enable **Pages → Source: GitHub Actions**.
- `wasm32-unknown-unknown` added per build (CI), already installed locally from
  task 001.

### Out of Scope
- No `workflow_dispatch` / manual trigger (see Future Improvements).
- No concurrency cancellation of in-flight deploys.
- No release/tag-based deployment, no custom domain.

### Verification
- Confirmed: a push to `main` triggers test → build → deploy successfully;
  published site reachable at the Pages URL.
