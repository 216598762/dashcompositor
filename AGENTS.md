# AGENTS.md

> Operating rules for AI assistants and human contributors working on
> `dashcompositor`. Read this file in full before submitting any change.

## 1. Project overview

`dashcompositor` is a **layer-based graphics compositor for the terminal**, written in
Rust. It maintains an in-memory stack of layers (sprites, text, images, shapes),
composites them into a single frame buffer, and projects the final image to the
terminal emulator via either:

- the **Kitty graphics protocol**, or
- **Sixel**,

whichever the host terminal supports. The project is an *output pipeline*, not a
terminal emulator; we never implement a TTY ourselves.

## 2. Hard rules (must not be violated)

1. **Library-first. Never reinvent the wheel.** If a capability we need already
   exists as a Rust crate, use it. Before writing or duplicating any non-trivial
   piece of logic, search the ecosystem.
2. **Start from [`rust-unofficial/awesome-rust`](https://github.com/rust-unofficial/awesome-rust).**
   Treat it as the default index for crate discovery. The
   [`Games`](https://github.com/rust-unofficial/awesome-rust#games),
   [`Graphics`](https://github.com/rust-unofficial/awesome-rust#graphics),
   [`Image processing`](https://github.com/rust-unofficial/awesome-rust#image-processing),
   [`Emulators`](https://github.com/rust-unofficial/awesome-rust#emulators), and
   [`Terminal`](https://github.com/rust-unofficial/awesome-rust#terminal) sections
   are the relevant starting points for this project.
3. **Commit and push regularly, with detailed descriptions.** A change is not
   "done" until it has been committed locally *and* pushed to `origin`. Commit
   messages must be multi-line: a short imperative subject plus a body that
   explains *what* and *why*. One-line messages like `fix` or `wip` are not
   acceptable here.
4. **Never open issues on this repository.** Do not use `gh issue create`, do not
   open issues through the web UI, do not file bugs against `dashcompositor`.
   Improvements go in via commits and pull requests, not issues. If you would
   normally file an issue, write it up as a note in the PR description or a
   section of the relevant doc instead.
5. **Stay in Rust.** No FFI shims to C/C++ unless a library we depend on already
   does so; do not introduce new language toolchains.

## 3. Library strategy

When picking dependencies, follow this hierarchy of search:

1. Search [**awesome-rust**](https://github.com/rust-unofficial/awesome-rust) and
   [`crates.io`](https://crates.io/) by capability.
2. Check whether an existing well-maintained crate already provides it. Prefer
   crates with active maintenance, a published crate, and clear licenses.
3. Only if no adequate crate exists after a real search should we consider
   writing the capability ourselves. Document *why* in the commit message in
   that case.

Concrete starting points for the project's known building blocks (these are
**hints, not requirements** — always verify current state on `crates.io` before
adopting):

| Need | Candidate crates to evaluate | Source |
| --- | --- | --- |
| Kitty graphics protocol client | `kittage`, `little-kitty`, `kitty-graphics-protocol` | crates.io |
| Sixel encoder | `icy_sixel`, `sixel`, `rasteroid` | crates.io |
| Protocol-agnostic terminal image output | `rasteroid` | crates.io |
| CPU 2D compositing / layer model | `tiny-skia` | crates.io |
| GPU 2D / compositor | `wgpu`, `pixels` | crates.io |
| Image decoding | `image` | crates.io |
| Terminal capability detection | `terminfo`, `crossterm`, `ratatui` (auxiliary only) | crates.io |

Always run `cargo search <crate>` and inspect the latest published version and
maintenance status before adding a dependency. Do not pin to a version known to
be older than six months without justification.

## 4. Commit & push cadence

- Commit at **logical milestones**: a new layer type, a new protocol backend, a
  refactor, a CI fix, a doc pass — not at every keystroke.
- Each commit must:
  - Have a subject line ≤ 72 characters, imperative mood.
  - Have a body that explains the *what* and *why* in prose.
  - Reference any design-doc section or AGENTS.md rule it implements/follows.
- **Push immediately after every commit.** Do not leave unpushed local commits
  behind. Agents especially: assume the user wants to see progress on `main`
  (or the active feature branch) in near real time.
- Keep commits small and focused. If a change spans unrelated concerns, split it.

## 5. Repository etiquette

- **Never** run `gh issue create`, `gh issue list` with intent to file, or open
  issues through any other channel.
- Use branches for non-trivial work; merge only after `cargo test` and `cargo
  clippy` pass.
- No secrets, no `.env` files, no credentials in commits.
- Do not rewrite history of `main`. Squash or rebase only on your own branch.

## 6. Definition of done for a change

Before pushing, a change is considered complete only when:

- `cargo build` (and `cargo build --release` when relevant) succeeds.
- `cargo test` passes for affected crates.
- `cargo clippy -- -D warnings` shows no new lints.
- `cargo fmt --check` passes.
- No unused dependencies, no dead code, no `unwrap()`/`expect()` in library
  code paths (tests and `examples/` can be more lenient).
- AGENTS.md, README.md, and any affected doc are kept in sync with reality.
- Changes are committed with a descriptive message *and* pushed to origin.

## 7. Compositor architecture (target shape)

Agents should design toward this model. Deviations need a written rationale in
the commit / PR.

```
        ┌──────────────┐
        │  Layer N     │  (image / text / sprite / shape)
        ├──────────────┤
        │  Layer ...   │
        ├──────────────┤
        │  Layer 1     │
        ├──────────────┤
        │  Layer 0     │  (background)
        └──────┬───────┘
               │  composite()
               ▼
        ┌──────────────┐
        │  FrameBuffer │  (RGBA / pixel grid)
        └──────┬───────┘
               │  encode()
               ▼
   ┌───────────────────────┐
   │ KitTy protocol encoder│   or
   ├───────────────────────┤
   │ Sixel     encoder     │
   └───────────────────────┘
```

Each layer implementation should be a Rust trait with concrete backends per
type. The framebuffer is a single contiguous pixel buffer. Output is selected
per terminal capability (Kitty first, Sixel fallback) and emitted as escape
sequences to `stdout`.

## 8. When unsure, ask

If a decision is not covered above — license choice, public API surface,
breaking-change policy — stop and ask the user before proceeding. The bar is
"small, correct, documented changes", not "move fast".
