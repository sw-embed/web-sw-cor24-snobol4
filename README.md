# web-sw-cor24-snobol4

Browser-based SNOBOL4 interpreter (Rust + Yew + WASM) running on an in-browser
COR24 emulator. See `docs/` for architecture, design, and plan.

## Embedded assets

The interpreter binary and demo programs are copied from
`../sw-cor24-snobol4` and embedded into the WASM bundle at compile time:

```sh
cp ../sw-cor24-snobol4/build/snobol4.bin assets/snobol4.bin
cp ../sw-cor24-snobol4/examples/{array,break,concat,count,hello,hello_goto,input,multiply,span,span_fail}.sno examples/
cp ../sw-cor24-snobol4/examples/input.dat examples/
```

`src/demos.rs` embeds these via `include_bytes!` / `include_str!` and exposes
a `DEMOS` table sorted alphabetically by display name.

## Develop

```sh
trunk serve   # http://localhost:9136
```
