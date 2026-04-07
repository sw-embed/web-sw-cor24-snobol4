//! Synchronous SNOBOL4 program runner.
//!
//! Mirrors `../sw-cor24-snobol4/scripts/run-snobol4.sh`:
//!   - SNOBOL4 interpreter binary loaded at 0x000000
//!   - SNOBOL4 source loaded at 0x080000
//!   - Optional input data loaded at 0x090000
//!   - Entry/PC = 0
//! Drains UART TX into a String when execution stops.

use cor24_emulator::{EmulatorCore, StopReason};

use crate::demos::SNOBOL4_BIN;

const SRC_ADDR: u32 = 0x080000;
const DATA_ADDR: u32 = 0x090000;

pub struct RunResult {
    pub output: String,
    pub instructions: u64,
    pub halted: bool,
    pub stop_reason: String,
}

/// Run a SNOBOL4 source program with an instruction budget.
pub fn run_snobol4(src: &str, data: &str, max_instrs: u64) -> Result<RunResult, String> {
    let mut emu = EmulatorCore::new();
    emu.set_uart_tx_busy_cycles(0); // instant TX in browser

    // Load interpreter at 0x0
    for (i, &b) in SNOBOL4_BIN.iter().enumerate() {
        emu.write_byte(i as u32, b);
    }
    // Load source at 0x080000
    for (i, &b) in src.as_bytes().iter().enumerate() {
        emu.write_byte(SRC_ADDR + i as u32, b);
    }
    // Null-terminate source
    emu.write_byte(SRC_ADDR + src.len() as u32, 0);

    // Load optional data at 0x090000
    if !data.is_empty() {
        for (i, &b) in data.as_bytes().iter().enumerate() {
            emu.write_byte(DATA_ADDR + i as u32, b);
        }
        emu.write_byte(DATA_ADDR + data.len() as u32, 0);
    }

    emu.set_pc(0);
    emu.resume();

    let mut total: u64 = 0;
    let mut halted = false;
    let mut stop_reason = String::from("budget exhausted");

    while total < max_instrs {
        let remaining = max_instrs - total;
        let batch = remaining.min(200_000);
        let result = emu.run_batch(batch);
        total += result.instructions_run as u64;

        match result.reason {
            StopReason::Halted => {
                halted = true;
                stop_reason = "halted".into();
                break;
            }
            StopReason::InvalidInstruction(byte) => {
                stop_reason = format!(
                    "invalid instruction 0x{:02X} at PC=0x{:06X}",
                    byte,
                    emu.pc()
                );
                break;
            }
            StopReason::Breakpoint(addr) => {
                stop_reason = format!("breakpoint at 0x{:06X}", addr);
                break;
            }
            StopReason::Paused => {
                stop_reason = "paused".into();
                break;
            }
            StopReason::StackOverflow(addr) => {
                stop_reason = format!("stack overflow at SP=0x{:06X}", addr);
                break;
            }
            StopReason::StackUnderflow(addr) => {
                stop_reason = format!("stack underflow at SP=0x{:06X}", addr);
                break;
            }
            StopReason::CycleLimit => {
                if result.instructions_run == 0 {
                    stop_reason = "stalled".into();
                    break;
                }
                // keep going until budget exhausted
            }
        }
    }

    Ok(RunResult {
        output: emu.get_uart_output().to_string(),
        instructions: total,
        halted,
        stop_reason,
    })
}
