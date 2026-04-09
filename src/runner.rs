//! SNOBOL4 program runner.
//!
//! Mirrors `../sw-cor24-snobol4/scripts/run-snobol4.sh`:
//!   - SNOBOL4 interpreter binary loaded at 0x000000
//!   - SNOBOL4 source loaded at 0x080000
//!   - Optional input data loaded at 0x090000
//!   - Entry/PC = 0
//!
//! `Session` exposes a tick-based interface so the UI can run long programs
//! in BATCH_SIZE chunks per animation frame without freezing the tab.

use cor24_emulator::{EmulatorCore, StopReason};

use crate::demos::SNOBOL4_BIN;

const SRC_ADDR: u32 = 0x080000;
const DATA_ADDR: u32 = 0x090000;

/// A live emulator running a SNOBOL4 program.
pub struct Session {
    emu: EmulatorCore,
    pub instructions: u64,
    pub done: bool,
    pub stop_reason: String,
    /// True if the program halted cleanly (HALT instruction).
    pub halted: bool,
    /// True if no data file was provided -- the interpreter probes
    /// 0x090000 at startup and switches READ_INPUT to live UART mode.
    interactive: bool,
}

/// Result of a single batched tick.
pub struct TickResult {
    pub instructions_run: u64,
    pub done: bool,
}

impl Session {
    /// Build a fresh session: load interpreter, source, optional data, set PC.
    pub fn new(src: &str, data: &str) -> Self {
        let mut emu = EmulatorCore::new();
        emu.set_uart_tx_busy_cycles(0);

        for (i, &b) in SNOBOL4_BIN.iter().enumerate() {
            emu.write_byte(i as u32, b);
        }
        for (i, &b) in src.as_bytes().iter().enumerate() {
            emu.write_byte(SRC_ADDR + i as u32, b);
        }
        emu.write_byte(SRC_ADDR + src.len() as u32, 0);
        if !data.is_empty() {
            for (i, &b) in data.as_bytes().iter().enumerate() {
                emu.write_byte(DATA_ADDR + i as u32, b);
            }
            emu.write_byte(DATA_ADDR + data.len() as u32, 0);
        }
        emu.set_pc(0);
        // SNOBOL4 manages its own stack/heap layout and uses SP outside the
        // strict 8 KB EBR window. Disable the emulator's stack-bounds check
        // (set_stack_bounds(0, 0)) so the interpreter can grow its stack
        // freely upward into SRAM, mirroring how the binary runs under
        // cor24-run with default settings. See docs/architecture.md for
        // a note on bumping SP to the top of SRAM if a deeper stack is
        // ever needed.
        emu.set_stack_bounds(0, 0);
        emu.resume();

        let interactive = data.is_empty();
        Self {
            emu,
            instructions: 0,
            done: false,
            stop_reason: String::new(),
            halted: false,
            interactive,
        }
    }

    /// Run up to `batch` instructions. Returns when the batch ends or the
    /// emulator stops for any reason.
    pub fn tick(&mut self, batch: u64) -> TickResult {
        if self.done {
            return TickResult {
                instructions_run: 0,
                done: true,
            };
        }
        let result = self.emu.run_batch(batch);
        self.instructions += result.instructions_run as u64;

        match result.reason {
            StopReason::Halted => {
                self.done = true;
                self.halted = true;
                self.stop_reason = "halted".into();
            }
            StopReason::InvalidInstruction(byte) => {
                self.done = true;
                self.stop_reason = format!(
                    "invalid instruction 0x{:02X} at PC=0x{:06X}",
                    byte,
                    self.emu.pc()
                );
            }
            StopReason::Breakpoint(addr) => {
                self.done = true;
                self.stop_reason = format!("breakpoint at 0x{:06X}", addr);
            }
            StopReason::Paused => {
                self.done = true;
                self.stop_reason = "paused".into();
            }
            StopReason::StackOverflow(addr) => {
                self.done = true;
                self.stop_reason = format!("stack overflow at SP=0x{:06X}", addr);
            }
            StopReason::StackUnderflow(addr) => {
                self.done = true;
                self.stop_reason = format!("stack underflow at SP=0x{:06X}", addr);
            }
            StopReason::CycleLimit => {
                if result.instructions_run == 0 {
                    self.done = true;
                    self.stop_reason = "stalled".into();
                }
            }
        }

        TickResult {
            instructions_run: result.instructions_run,
            done: self.done,
        }
    }

    pub fn output(&self) -> &str {
        self.emu.get_uart_output()
    }

    /// Inject a byte into the emulated UART RX register. The SNOBOL4
    /// interpreter's `READ_INPUT` polls UART RX in TTY mode (when no data
    /// file is loaded at 0x090000), so this is how the browser feeds live
    /// keystrokes to a running program.
    pub fn send_input_byte(&mut self, byte: u8) {
        self.emu.send_uart_byte(byte);
    }

    /// True if the session was started without a data file -- in that case
    /// the interpreter is in TTY mode and `send_input_byte` is meaningful.
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }
}
