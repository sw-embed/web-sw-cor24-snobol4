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
    /// Bytes queued for injection into the UART RX register. The RX
    /// register has FIFO depth 1, so calling `send_uart_byte` in a tight
    /// loop overwrites earlier bytes. Instead we drain one byte per
    /// `tick()` -- that gives the interpreter a full instruction batch
    /// to poll/consume each byte before we hand it the next one.
    rx_queue: std::collections::VecDeque<u8>,
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
            rx_queue: std::collections::VecDeque::new(),
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
        // Drain at most one queued RX byte per tick. The previous tick
        // ran a full instruction batch (~200k instrs), which is far
        // more than enough for SNOBOL's READ_INPUT polling loop to
        // consume the byte we handed it last time. Sending one byte
        // per tick respects the UART RX register's depth-1 semantics
        // without needing to poll uart_rx_ready (which isn't exported).
        if let Some(byte) = self.rx_queue.pop_front() {
            self.emu.send_uart_byte(byte);
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

    /// Queue a byte for delivery to the emulated UART RX register.
    /// Bytes drain one-per-tick (see `tick`) so a full typed line
    /// arrives without overwriting itself in the depth-1 RX register.
    pub fn send_input_byte(&mut self, byte: u8) {
        self.rx_queue.push_back(byte);
    }

    /// True if the session was started without a data file -- in that case
    /// the interpreter is in TTY mode and `send_input_byte` is meaningful.
    pub fn is_interactive(&self) -> bool {
        self.interactive
    }
}
