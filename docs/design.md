# Design: web-sw-cor24-snobol4 UI

Deliberately simpler than the macrolisp REPL. No split view, no live trace,
no prelude tiers — SNOBOL4 programs are batch programs, so the UI is a
batch-style "edit, run, see output" page.

## Layout (single column, max-width ~960px)

```
┌──────────────────────────────────────────────────────────┐
│ web-sw-cor24-snobol4              [example ▼]  [ Run ]   │
├──────────────────────────────────────────────────────────┤
│ source (.sno)                                            │
│ ┌──────────────────────────────────────────────────────┐ │
│ │        OUTPUT = 'Hello, World!'                      │ │
│ │ END                                                  │ │
│ └──────────────────────────────────────────────────────┘ │
│ input data (optional, loaded at 0x090000)                │
│ ┌──────────────────────────────────────────────────────┐ │
│ │                                                      │ │
│ └──────────────────────────────────────────────────────┘ │
│ ─ output ──────────────────── status: idle ── 0 instrs ─ │
│ ┌──────────────────────────────────────────────────────┐ │
│ │ Hello, World!                                        │ │
│ └──────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────┘
```

## Controls

- **Demo dropdown**: replaces source (and clears output) with the chosen
  bundled `.sno`. The dropdown lists exactly the demos from the
  `sw-cor24-snobol4` justfile `demos` recipe, **in alphabetical order**:

  1. array
  2. break
  3. concat
  4. count
  5. hello
  6. hello-goto
  7. input
  8. multiply
  9. span
  10. span-fail

  Default selection on load is `hello`. The `input` demo also pre-fills the
  input-data textarea with `examples/input.dat`.
- **Run**: starts a batched run. While running, the button becomes **Stop**.
- **Reset** (small link): clears output and resets the emulator.
- **Status line**: shows `idle | running… | done (N instrs, T ms) | halted | error: …`.

## Color / typography

Monospace everything in the source/data/output panels. System sans for
chrome. Dark theme by default (matches macrolisp project), single CSS file.

## Keyboard

- `Ctrl/Cmd+Enter` in source area: Run.
- `Esc` while running: Stop.

## Error display

Anything that fails before the emulator starts (e.g. assembler/loader error)
is shown in the output panel prefixed with `error:` and the status line goes
red. Halt for "out of budget" shows status `halted (budget)` and surfaces a
"Increase budget" link that bumps `max_instrs` 4× and re-runs.

## State machine

```
Idle ──Run──▶ Running ──tick (budget left)──▶ Running
                │                                 │
                │                       (uart halt│emulator stop)
                │                                 ▼
                │                              Done
                ├──Stop──▶ Stopped
                └──error──▶ Error
```

## Out of scope for v1

- Syntax highlighting (plain `<textarea>`).
- Saving snippets to localStorage.
- Sharing via URL hash (nice-to-have follow-up).
- Trace / register / memory inspectors.
