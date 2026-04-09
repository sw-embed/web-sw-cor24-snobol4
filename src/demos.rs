//! Bundled SNOBOL4 demo programs and the embedded interpreter binary.

/// Pre-built SNOBOL4 interpreter binary (COR24 image).
pub static SNOBOL4_BIN: &[u8] = include_bytes!("../assets/snobol4.bin");

/// A bundled demo program.
pub struct Demo {
    /// Display name shown in the UI dropdown.
    pub name: &'static str,
    /// SNOBOL4 source code.
    pub source: &'static str,
    /// Optional input data preloaded into the data textarea.
    pub data: Option<&'static str>,
}

/// All bundled demos, sorted alphabetically by display name.
pub static DEMOS: &[Demo] = &[
    Demo {
        name: "array",
        source: include_str!("../examples/array.sno"),
        data: None,
    },
    Demo {
        name: "break",
        source: include_str!("../examples/break.sno"),
        data: None,
    },
    Demo {
        name: "concat",
        source: include_str!("../examples/concat.sno"),
        data: None,
    },
    Demo {
        name: "count",
        source: include_str!("../examples/count.sno"),
        data: None,
    },
    // Eliza ships with no data so the runner picks "interactive UART"
    // mode -- the UI then shows a stdin field while the program waits
    // on READ_INPUT. The canned eliza.dat is still bundled as the
    // separate "eliza-batch" entry for users who want the non-REPL
    // transcript or want to test on a CI run.
    Demo {
        name: "eliza",
        source: include_str!("../examples/eliza.sno"),
        data: None,
    },
    Demo {
        name: "eliza-batch",
        source: include_str!("../examples/eliza.sno"),
        data: Some(include_str!("../examples/eliza.dat")),
    },
    Demo {
        name: "factorial",
        source: include_str!("../examples/factorial.sno"),
        data: None,
    },
    Demo {
        name: "fibonacci",
        source: include_str!("../examples/fibonacci.sno"),
        data: None,
    },
    Demo {
        name: "fizzbuzz",
        source: include_str!("../examples/fizzbuzz.sno"),
        data: None,
    },
    Demo {
        name: "gcd",
        source: include_str!("../examples/gcd.sno"),
        data: None,
    },
    Demo {
        name: "hello",
        source: include_str!("../examples/hello.sno"),
        data: None,
    },
    Demo {
        name: "hello-goto",
        source: include_str!("../examples/hello_goto.sno"),
        data: None,
    },
    Demo {
        name: "input",
        source: include_str!("../examples/input.sno"),
        data: Some(include_str!("../examples/input.dat")),
    },
    Demo {
        name: "multiply",
        source: include_str!("../examples/multiply.sno"),
        data: None,
    },
    Demo {
        name: "n-queens",
        source: include_str!("../examples/nqueens.sno"),
        data: None,
    },
    Demo {
        name: "palindrome",
        source: include_str!("../examples/palindrome.sno"),
        data: None,
    },
    Demo {
        name: "reverse",
        source: include_str!("../examples/reverse.sno"),
        data: None,
    },
    Demo {
        name: "sieve",
        source: include_str!("../examples/sieve.sno"),
        data: None,
    },
    Demo {
        name: "span",
        source: include_str!("../examples/span.sno"),
        data: None,
    },
    Demo {
        name: "span-fail",
        source: include_str!("../examples/span_fail.sno"),
        data: None,
    },
];
