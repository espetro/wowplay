//! x87 instruction corpus for testing translation correctness.
//!
//! Provides a comprehensive set of x87 instruction byte patterns
//! that represent the most common operations found in WoW 3.3.5a.
//! Each entry includes the raw bytes, a description, and expected behavior.

/// A single x87 instruction pattern for testing.
pub struct X87Instruction {
    /// Raw instruction bytes
    pub bytes: &'static [u8],
    /// Human-readable description
    pub description: &'static str,
    /// Expected result type/category
    pub expected_result: &'static str,
}

/// Comprehensive corpus of x87 instruction patterns.
///
/// Covers the full spectrum of x87 operations used in 32-bit Windows
/// applications, particularly games like WoW 3.3.5a.
pub const X87_CORPUS: &[X87Instruction] = &[
    // Basic arithmetic
    X87Instruction {
        bytes: &[0xD8, 0xC1], // FADD ST(0), ST(1)
        description: "Add ST(1) to ST(0)",
        expected_result: "addition",
    },
    X87Instruction {
        bytes: &[0xDC, 0xC1], // FADD ST(1), ST(0)
        description: "Add ST(0) to ST(1)",
        expected_result: "addition",
    },
    X87Instruction {
        bytes: &[0xD8, 0xE1], // FSUB ST(0), ST(1)
        description: "Subtract ST(1) from ST(0)",
        expected_result: "subtraction",
    },
    X87Instruction {
        bytes: &[0xDC, 0xE9], // FSUB ST(1), ST(0)
        description: "Subtract ST(0) from ST(1)",
        expected_result: "subtraction",
    },
    X87Instruction {
        bytes: &[0xD8, 0xC9], // FMUL ST(0), ST(1)
        description: "Multiply ST(0) by ST(1)",
        expected_result: "multiplication",
    },
    X87Instruction {
        bytes: &[0xDC, 0xC9], // FMUL ST(1), ST(0)
        description: "Multiply ST(1) by ST(0)",
        expected_result: "multiplication",
    },
    X87Instruction {
        bytes: &[0xD8, 0xF1], // FDIV ST(0), ST(1)
        description: "Divide ST(0) by ST(1)",
        expected_result: "division",
    },
    X87Instruction {
        bytes: &[0xDC, 0xF9], // FDIV ST(1), ST(0)
        description: "Divide ST(1) by ST(0)",
        expected_result: "division",
    },
    X87Instruction {
        bytes: &[0xD8, 0xF9], // FDIVR ST(0), ST(1)
        description: "Reverse divide ST(0) by ST(1)",
        expected_result: "division",
    },
    // Trigonometric
    X87Instruction {
        bytes: &[0xD9, 0xFE], // FSIN
        description: "Sine of ST(0)",
        expected_result: "trigonometric",
    },
    X87Instruction {
        bytes: &[0xD9, 0xFF], // FCOS
        description: "Cosine of ST(0)",
        expected_result: "trigonometric",
    },
    X87Instruction {
        bytes: &[0xD9, 0xFB], // FSINCOS
        description: "Sine and cosine of ST(0)",
        expected_result: "trigonometric",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF2], // FPTAN
        description: "Partial tangent of ST(0)",
        expected_result: "trigonometric",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF3], // FPATAN
        description: "Partial arctangent of ST(1)/ST(0)",
        expected_result: "trigonometric",
    },
    // Square root and logarithmic
    X87Instruction {
        bytes: &[0xD9, 0xFA], // FSQRT
        description: "Square root of ST(0)",
        expected_result: "sqrt",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF1], // Fyl2x
        description: "ST(1) * log2(ST(0))",
        expected_result: "logarithmic",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF0], // F2XM1
        description: "2^ST(0) - 1",
        expected_result: "exponential",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF4], // FXTRACT
        description: "Extract exponent and significand",
        expected_result: "decomposition",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF5], // FPREM1
        description: "Partial remainder (IEEE)",
        expected_result: "remainder",
    },
    // Comparison
    X87Instruction {
        bytes: &[0xD8, 0xD1], // FCOM ST(1)
        description: "Compare ST(0) with ST(1)",
        expected_result: "comparison",
    },
    X87Instruction {
        bytes: &[0xD8, 0xD9], // FCOMP ST(1)
        description: "Compare ST(0) with ST(1) and pop",
        expected_result: "comparison",
    },
    X87Instruction {
        bytes: &[0xDF, 0xF0], // FCOMI ST(0), ST(0)
        description: "Compare ST(0) with ST(0) and set EFLAGS",
        expected_result: "comparison",
    },
    X87Instruction {
        bytes: &[0xDB, 0xF0], // FCOMI ST(0), ST(0) (alternate)
        description: "Compare ST(0) with ST(0) and set EFLAGS",
        expected_result: "comparison",
    },
    X87Instruction {
        bytes: &[0xD9, 0xE4], // FTST
        description: "Test ST(0) against zero",
        expected_result: "comparison",
    },
    X87Instruction {
        bytes: &[0xD9, 0xE5], // FXAM
        description: "Examine ST(0)",
        expected_result: "examination",
    },
    // Stack manipulation
    X87Instruction {
        bytes: &[0xD9, 0xC9], // FXCH ST(1)
        description: "Exchange ST(0) and ST(1)",
        expected_result: "stack_exchange",
    },
    X87Instruction {
        bytes: &[0xDD, 0xD8], // FSTP ST(0)
        description: "Store and pop ST(0)",
        expected_result: "stack_pop",
    },
    X87Instruction {
        bytes: &[0xD9, 0xE8], // FLD1
        description: "Load 1.0 onto stack",
        expected_result: "constant_load",
    },
    X87Instruction {
        bytes: &[0xD9, 0xEE], // FLDZ
        description: "Load 0.0 onto stack",
        expected_result: "constant_load",
    },
    X87Instruction {
        bytes: &[0xD9, 0xED], // FLDLN2
        description: "Load ln(2) onto stack",
        expected_result: "constant_load",
    },
    X87Instruction {
        bytes: &[0xD9, 0xEC], // FLDLG2
        description: "Load log10(2) onto stack",
        expected_result: "constant_load",
    },
    X87Instruction {
        bytes: &[0xD9, 0xEB], // FLDPI
        description: "Load pi onto stack",
        expected_result: "constant_load",
    },
    X87Instruction {
        bytes: &[0xDD, 0xD9], // FSTP ST(1)
        description: "Store and pop ST(1)",
        expected_result: "stack_pop",
    },
    // Memory operations
    X87Instruction {
        bytes: &[0xD9, 0x00], // FLD DWORD PTR [EAX]
        description: "Load float from memory [EAX]",
        expected_result: "memory_load",
    },
    X87Instruction {
        bytes: &[0xDD, 0x00], // FLD QWORD PTR [EAX]
        description: "Load double from memory [EAX]",
        expected_result: "memory_load",
    },
    X87Instruction {
        bytes: &[0xD9, 0x10], // FST DWORD PTR [EAX]
        description: "Store float to memory [EAX]",
        expected_result: "memory_store",
    },
    X87Instruction {
        bytes: &[0xDD, 0x10], // FST QWORD PTR [EAX]
        description: "Store double to memory [EAX]",
        expected_result: "memory_store",
    },
    X87Instruction {
        bytes: &[0xDB, 0x00], // FILD DWORD PTR [EAX]
        description: "Load integer from memory [EAX]",
        expected_result: "integer_load",
    },
    X87Instruction {
        bytes: &[0xDF, 0x00], // FILD QWORD PTR [EAX]
        description: "Load long integer from memory [EAX]",
        expected_result: "integer_load",
    },
    X87Instruction {
        bytes: &[0xDB, 0x10], // FIST DWORD PTR [EAX]
        description: "Store integer to memory [EAX]",
        expected_result: "integer_store",
    },
    X87Instruction {
        bytes: &[0xDF, 0x10], // FISTP QWORD PTR [EAX]
        description: "Store long integer and pop",
        expected_result: "integer_store",
    },
    // Control word operations
    X87Instruction {
        bytes: &[0xD9, 0x3C, 0x24], // FNSTCW [ESP]
        description: "Store control word to [ESP]",
        expected_result: "control_word",
    },
    X87Instruction {
        bytes: &[0xD9, 0x6C, 0x24, 0x02], // FLDCW [ESP+2]
        description: "Load control word from [ESP+2]",
        expected_result: "control_word",
    },
    X87Instruction {
        bytes: &[0x9B, 0xD9, 0x3C, 0x24], // FSTCW [ESP]
        description: "Store control word to [ESP] (wait)",
        expected_result: "control_word",
    },
    // Additional common patterns
    X87Instruction {
        bytes: &[0xD9, 0xE1], // FABS
        description: "Absolute value of ST(0)",
        expected_result: "arithmetic",
    },
    X87Instruction {
        bytes: &[0xD9, 0xE0], // FCHS
        description: "Change sign of ST(0)",
        expected_result: "arithmetic",
    },
    X87Instruction {
        bytes: &[0xD9, 0xFC], // FRNDINT
        description: "Round ST(0) to integer",
        expected_result: "rounding",
    },
    X87Instruction {
        bytes: &[0xD9, 0xFD], // FSCALE
        description: "Scale ST(0) by ST(1)",
        expected_result: "exponential",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF7], // FINCSTP
        description: "Increment stack pointer",
        expected_result: "stack_manip",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF6], // FDECSTP
        description: "Decrement stack pointer",
        expected_result: "stack_manip",
    },
    X87Instruction {
        bytes: &[0xDD, 0xE1], // FUCOM ST(1)
        description: "Unordered compare ST(0) with ST(1)",
        expected_result: "comparison",
    },
    X87Instruction {
        bytes: &[0xDD, 0xE9], // FUCOMP ST(1)
        description: "Unordered compare and pop",
        expected_result: "comparison",
    },
    X87Instruction {
        bytes: &[0xDA, 0xC0], // FCMOVB ST(0), ST(0)
        description: "Conditional move if below",
        expected_result: "conditional",
    },
    X87Instruction {
        bytes: &[0xDA, 0xC8], // FCMOVE ST(0), ST(0)
        description: "Conditional move if equal",
        expected_result: "conditional",
    },
    X87Instruction {
        bytes: &[0xDA, 0xD0], // FCMOVBE ST(0), ST(0)
        description: "Conditional move if below or equal",
        expected_result: "conditional",
    },
    X87Instruction {
        bytes: &[0xDA, 0xD8], // FCMOVU ST(0), ST(0)
        description: "Conditional move if unordered",
        expected_result: "conditional",
    },
    X87Instruction {
        bytes: &[0xDB, 0xE0], // FNINIT
        description: "Initialize FPU without check",
        expected_result: "initialization",
    },
    X87Instruction {
        bytes: &[0xD9, 0xE3], // FNINIT (alternate encoding)
        description: "Initialize FPU",
        expected_result: "initialization",
    },
    X87Instruction {
        bytes: &[0xD9, 0xF8], // FPREM
        description: "Partial remainder",
        expected_result: "remainder",
    },
    X87Instruction {
        bytes: &[0xD9, 0xEA], // FSQRT (alternate context)
        description: "Square root",
        expected_result: "sqrt",
    },
];

/// Get the number of instructions in the corpus.
pub fn corpus_size() -> usize {
    X87_CORPUS.len()
}

/// Get an instruction by index.
pub fn get_instruction(index: usize) -> Option<&'static X87Instruction> {
    X87_CORPUS.get(index)
}

/// Get all instructions matching a specific expected result type.
pub fn get_by_result_type(result_type: &str) -> Vec<&'static X87Instruction> {
    X87_CORPUS
        .iter()
        .filter(|inst| inst.expected_result == result_type)
        .collect()
}
