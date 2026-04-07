# Architecture: web-sw-cor24-snobol4

## Decision: Rust/Yew + cor24-emulator + pre-built snobol4.bin

We do **not** rebuild the SNOBOL4 interpreter in the browser. Instead the
SNOBOL4 interpreter binary (`snobol4.bin`) is built ahead of time by
`../sw-cor24-snobol4` and embedded in the WASM bundle. At runtime the browser
loads it into the COR24 emulator and feeds the user's `.sno` source through the
same memory layout the host `run-snobol4.sh` script uses.

This mirrors the approach taken by `../web-sw-cor24-macrolisp`: ship the
emulator + a frozen interpreter image, keep the frontend thin.

### Why not Emscripten / pure-Rust SNOBOL4?

Same reasoning as the macrolisp project. Users want to see SNOBOL4 *running on
COR24*. The cor24-emulator crate already runs in the browser and is the same
emulator used by every other tool in the stack.

## Pipeline

```
Build time (out of band, in sibling repo):
  src/*.plsw  --[plsw compiler]-->  build/snobol4.s  --[cor24 asm]-->  build/snobol4.bin

Build time (this repo, via build.rs or trunk pre-build hook):
  ../sw-cor24-snobol4/build/snobol4.bin  --copy-->  assets/snobol4.bin
  examples/*.sno                          --embed--> Rust string table

Browser runtime:
  snobol4.bin   -> load at 0x000000, entry = 0
  user .sno src -> load at 0x080000
  user .dat (opt) -> load at 0x090000
  EmulatorCore.run(...) until halt or instruction-budget
  UART TX bytes -> output panel
```

## Stack

- **Frontend**: Yew 0.21 (CSR), `wasm-bindgen`, `web-sys`, `gloo`.
- **Build tool**: Trunk.
- **CPU**: `cor24-emulator` crate (path dep on `../sw-cor24-emulator`).
- **ISA types**: `cor24-isa` crate.
- **Interpreter binary**: pre-built `snobol4.bin` from `../sw-cor24-snobol4`,
  embedded with `include_bytes!`.
- **Examples**: bundled `.sno` files, embedded with `include_str!`.

## Project structure

```
web-sw-cor24-snobol4/
├── Cargo.toml
├── Trunk.toml
├── index.html
├── assets/
│   └── snobol4.bin           # copied from ../sw-cor24-snobol4/build/
├── examples/                 # mirrored from ../sw-cor24-snobol4/examples/
│   ├── array.sno
│   ├── break.sno
│   ├── concat.sno
│   ├── count.sno
│   ├── hello.sno
│   ├── hello_goto.sno
│   ├── input.sno
│   ├── input.dat
│   ├── multiply.sno
│   ├── span.sno
│   └── span_fail.sno
├── src/
│   ├── main.rs               # Yew renderer entry
│   ├── lib.rs                # App component
│   ├── runner.rs             # Emulator wrapper: load+run+collect UART
│   ├── examples.rs           # Generated/static example table
│   └── ui.css
├── pages/                    # committed Trunk dist output for GH Pages
├── docs/
└── .github/workflows/pages.yml
```

## Component design

### `App`
Top-level Yew component holding state: selected example, source text, input
data text, output text, status (`Idle | Running | Done { instrs, ms } | Error`),
instruction-budget knob.

### `runner.rs`
Stateless helper:
```rust
pub struct RunResult { pub output: String, pub instrs: u64, pub halted: bool }
pub fn run_snobol4(src: &str, data: Option<&str>, max_instrs: u64) -> Result<RunResult, String>;
```
Internally:
1. `Assembler`-free path: just `EmulatorCore::new()` and load binaries via the
   emulator's `load_bytes(addr, &[u8])` API.
2. Load `SNOBOL4_BIN` at 0x0, src bytes at 0x080000, optional data at 0x090000.
3. Set entry / PC to 0, run with budget, drain UART TX.

To keep the UI responsive on long programs, runs happen in a `Timeout`-driven
batch loop (BATCH_SIZE instructions per animation frame), the same trick the
macrolisp REPL uses.

### Examples table
A `build.rs` (or a small `xtask`) walks `examples/*.sno` and emits a
`pub static EXAMPLES: &[(&str, &str)]` so adding an example is just dropping a
file. For v1 we can hand-maintain the table to keep `build.rs` out of the
critical path.

## GitHub Pages deployment

Same pattern as `../web-sw-cor24-macrolisp`:

1. Developer runs `trunk build --release` locally.
2. Output is copied into `pages/` and committed.
3. `.github/workflows/pages.yml` uploads `./pages` as the Pages artifact on
   every push to `main`.

This avoids needing Rust/Trunk in CI and keeps the workflow trivial.

## Stack pointer & memory layout

COR24 has a small 8 KB Embedded Block RAM window at `0xFEE000–0xFEFFFF`
that the default hardware stack lives in (initial SP = `0xFEEC00`, 3 KB
populated; `0xFF0000` for the full 8 KB). The SNOBOL4 interpreter manages
its own stack/heap layout and uses SP outside the strict EBR window, so
`runner::Session::new` calls `emu.set_stack_bounds(0, 0)` to disable the
emulator's bounds check.

**If a program ever needs more than 8 KB of stack**, the SP can be
initialized to the **top of main SRAM**, above the upward-growing heap.
SNOBOL4's heap grows up from a low SRAM base, so a stack growing down
from the top of SRAM (e.g. just below `0xFEE000`, where the EBR window
begins) gives effectively the entire SRAM region as a stack arena while
keeping heap and stack from colliding until they meet in the middle —
the same trick traditional Unix processes use. This is not done by
default; bump SP in `Session::new` if/when needed.
