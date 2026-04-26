# CLAUDE.md — engineering guardrails for `tun2proxy`

These rules apply to every change in this repo. They are enforced; do not
work around them. `AGENTS.md` is a verbatim mirror for non-Claude agents
(Codex, GitHub Copilot CLI, etc.) — keep the two files in sync.

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
- No `cargo clippy ... --warn` to silence findings — fix or
  `#[allow(...)]` with a comment explaining why.

## 4. Code rules

- `#![forbid(unsafe_code)]` in every crate **except** `tun2proxy-ffi`
  (which exposes a C ABI and needs `unsafe extern "C"`).
- MSRV: `1.85` (declared in workspace `Cargo.toml`). Do not use features
  newer than MSRV without bumping it deliberately in its own PR.
- Prefer maintained crates from crates.io over vendoring. Pin minor
  versions in `[workspace.dependencies]`.
- Errors: `thiserror` for library types, `anyhow` for the CLI binary
  only. No `unwrap()`/`expect()` on runtime paths — only in tests and
  `const`-context initialization.
- Comments: explain *why*, not *what*. The compiler already knows what.

## 5. Docs sync

When a milestone ships, the same PR must:

- Move the milestone row in `docs/ROADMAP.md` from "Planned" to "Done"
  (with the merge commit hash).
- Strike off the corresponding items in `docs/TODO.md`.
- Update `README.md` status badges if the public API gained or lost
  anything.

## 6. Scope discipline

- A bug fix fixes the bug. It does not refactor surrounding code,
  reorganize modules, or add unrelated features. If you spot adjacent
  cleanup, file a TODO and propose a follow-up PR.
- Do not introduce abstractions for hypothetical future requirements.
  Three similar lines beat a premature trait.

## 7. Platform scope (v1)

Supported: macOS, Linux, Android (FFI), iOS (FFI). Windows / Wintun
support is explicitly out of scope until v2 — do not add `#[cfg(windows)]`
branches for it; if you need to, raise the question first.

## 8. Reference projects (study, do not vendor)

- `mihomo-android` (Go + Rust): single-owner Stack with mpsc fan-in.
  Path: `~/workspace/mihomo-android/core/src/main/rust/mihomo-android-ffi`.
- `blechschmidt/tun2proxy`: prior Rust art using `ipstack` instead of
  `netstack-smoltcp`. Useful for CLI ergonomics and FFI shape.
