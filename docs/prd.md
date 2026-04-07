# PRD: web-sw-cor24-snobol4

## Goal

A zero-install, browser-based way to **write and run SNOBOL4 programs** on the
COR24 virtual machine. Users land on a GitHub Pages URL and within seconds can
edit a `.sno` example, hit Run, and see output.

## Audience

- Retro-computing / esolang enthusiasts curious about SNOBOL4.
- Students or instructors who want a no-setup SNOBOL4 sandbox.
- Anyone evaluating the COR24 / PL-SW / SNOBOL4 toolchain stack.

## Non-goals (initial release)

- Editing or assembling COR24 programs directly.
- Loading external `.dat` files via filesystem (data is provided inline as a
  separate text area).
- Live execution trace / register viewer (the macrolisp UI has this; we keep
  the SNOBOL4 UI deliberately simpler for v1).
- Multi-user state, persistence to a backend, or accounts.

## User stories

1. *As a visitor* I see a SNOBOL4 source area pre-filled with `hello.sno`, an
   optional input data area, a **Run** button, and an output area.
2. *As a visitor* I can pick from a **demo dropdown** whose entries match the
   `demos` recipe in `../sw-cor24-snobol4/justfile`, listed in alphabetical
   order (`array`, `break`, `concat`, `count`, `hello`, `hello-goto`, `input`,
   `multiply`, `span`, `span-fail`). Selecting an entry replaces the source
   area with that bundled `.sno`.
3. *As a visitor* I click Run and within a few seconds see the SNOBOL4 program's
   UART output. Long-running programs surface a status indicator and can be
   stopped.
4. *As a visitor* I see instruction count and elapsed time after each run.
5. *As a developer* I can `trunk serve` locally and iterate on the UI without
   rebuilding the SNOBOL4 interpreter.

## Success criteria for v1

- Live demo published at `https://<user>.github.io/web-sw-cor24-snobol4/`.
- All bundled examples run to completion in the browser and produce the same
  UART output as `just run` on the host.
- Cold-load to first successful Run on a typical laptop is under 5 seconds.
- Repository builds with `trunk build --release` from a clean checkout, given
  sibling repos `sw-cor24-snobol4` and `sw-cor24-emulator`.
