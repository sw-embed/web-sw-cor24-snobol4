# Implementation plan

The agentrail saga (`.agentrail/saga.toml`) is the source of truth for
ordering. This document is the human-readable overview.

## Phase 0 — Skeleton
1. Scaffold Cargo project (`cdylib + rlib`), `Trunk.toml`, `index.html`.
2. Add path deps on `cor24-emulator` and `cor24-isa`.
3. Hello-world Yew `App` that just renders the page chrome.
4. `trunk serve` works locally.

## Phase 1 — Run a fixed program
5. Copy `../sw-cor24-snobol4/build/snobol4.bin` into `assets/`.
6. Implement `runner::run_snobol4` (synchronous, no batching) running
   `examples/hello.sno`.
7. Wire a single hard-coded **Run hello** button to `runner::run_snobol4` and
   display output.

## Phase 2 — Real UI
8. Source `<textarea>`, optional data `<textarea>`, output `<pre>`, status
   line, Run button.
9. Bundled demos table + dropdown — exactly the demos listed in the
   `sw-cor24-snobol4` justfile `demos` recipe, **in alphabetical order**:
   `array`, `break`, `concat`, `count`, `hello`, `hello-goto`, `input`,
   `multiply`, `span`, `span-fail`. Default = `hello`. The `input` demo also
   loads `examples/input.dat` into the data textarea.
10. Convert runner to batched-tick mode (BATCH_SIZE instructions per
    animation frame) so long runs don't freeze the tab; add Stop.

## Phase 3 — Polish
11. Status line counters (instructions, elapsed ms), Ctrl/Cmd+Enter shortcut.
12. Error panel + "Increase budget" affordance.
13. Dark-theme CSS pass.

## Phase 4 — Ship
14. `trunk build --release` → copy `dist/` to `pages/`.
15. `.github/workflows/pages.yml` (mirror of macrolisp project).
16. Enable Pages in repo settings; verify live URL.
17. Update `README.md` with screenshot + live link.

## Risks / open questions

- **Emulator API surface**: confirm `cor24-emulator` exposes `load_bytes` and
  a way to set entry PC from outside `Assembler`. Fall back to assembling a
  tiny shim if not.
- **Binary size**: `snobol4.bin` size + WASM bundle should stay under a few
  MB; if not, gzip in `pages/` and let GH Pages serve `.gz`.
- **Trace feature**: deferred to v2. Architecture leaves room (the macrolisp
  REPL already shows how to wire `TraceBuffer`).
