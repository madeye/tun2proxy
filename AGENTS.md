# AGENTS.md — engineering guardrails for `tun2proxy`

> Mirror of `CLAUDE.md`. Read by non-Claude agents (Codex, Copilot CLI,
> etc.). When `CLAUDE.md` changes, update this file in the same commit.

These rules apply to every change in this repo. They are enforced; do
not work around them.

## 1. Branching

- **Never commit to `main`.** Every change starts on a new branch.
- Branch naming: `feature/<slug>`, `fix/<slug>`, `chore/<slug>`,
  `docs/<slug>`, `release/<version>`.
- One PR per logical feature. Match the milestones in
  [`docs/ROADMAP.md`](docs/ROADMAP.md); do not bundle unrelated work.

## 2. Pre-push gate (mandatory, no exceptions)

Before every `git push`, run locally and confirm all three pass:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

If any step fails, fix the underlying cause and re-run. Pushing a commit
that has not been verified locally is a hard violation.

## 3. Forbidden shortcuts

- No `git commit --no-verify`. No skipping hooks for any reason.
- No `--no-gpg-sign` unless the user explicitly asks.
- No force-pushing to `main`. Force-pushing to your own feature branch
  is fine when needed.
- No silencing clippy findings without a comment explaining why.

## 4. Code rules

- `#![forbid(unsafe_code)]` in every crate **except** `tun2proxy-ffi`.
- MSRV: `1.85`. Do not use features newer than MSRV without bumping it
  deliberately in its own PR.
- Prefer maintained crates from crates.io over vendoring. Pin minor
  versions in `[workspace.dependencies]`.
- Errors: `thiserror` for libraries, `anyhow` for the CLI binary only.
  No `unwrap()`/`expect()` on runtime paths — only in tests and
  `const`-context initialization.
- Comments: explain *why*, not *what*.

## 5. Docs sync

When a milestone ships, the same PR must:

- Move the milestone row in `docs/ROADMAP.md` from "Planned" to "Done"
  (with the merge commit hash).
- Strike off the corresponding items in `docs/TODO.md`.
- Update `README.md` status badges if the public API gained or lost
  anything.

## 6. Scope discipline

- A bug fix fixes the bug — it does not refactor surrounding code or
  add unrelated features.
- Do not introduce abstractions for hypothetical future requirements.

## 7. Platform scope (v1)

Supported: macOS, Linux, Android (FFI), iOS (FFI). Windows / Wintun is
out of scope until v2.

## 8. Reference projects (study, do not vendor)

- `mihomo-android`: single-owner Stack with mpsc fan-in.
- `blechschmidt/tun2proxy`: prior Rust art with `ipstack`.
