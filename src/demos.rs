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
    Demo {
        name: "fizzbuzz",
        source: include_str!("../examples/fizzbuzz.sno"),
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
