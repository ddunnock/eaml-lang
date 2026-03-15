//! Error code definitions for the EAML compiler.
//!
//! All 38 error codes from spec/ERRORS.md plus 4 new lexer-specific codes
//! from the reserved SYN001-039 range.

/// All diagnostic codes emitted by the EAML compiler.
///
/// Codes follow the pattern `PREFIX + NNN` where PREFIX is a 2-3 letter
/// category identifier and NNN is a zero-padded numeric code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // === SYN: Syntax errors -- lexer (new, from reserved SYN001-039 range) ===
    /// SYN001: Unexpected/unrecognized character
    Syn001,
    /// SYN002: Unterminated string literal
    Syn002,
    /// SYN003: Unterminated block comment
    Syn003,
    /// SYN004: Invalid escape sequence in string literal
    Syn004,

    // === SYN: Syntax errors -- lexer (spec-defined) ===
    /// SYN042: Unterminated string literal (spec-defined)
    Syn042,
    /// SYN043: Invalid escape sequence
    Syn043,
    /// SYN044: Unexpected token
    Syn044,
    /// SYN045: Unclosed template string interpolation
    Syn045,
    /// SYN046: Unclosed Python bridge block
    Syn046,

    // === SYN: Syntax errors -- parser ===
    /// SYN050: Expected token not found
    Syn050,
    /// SYN060: Invalid declaration
    Syn060,
    /// SYN061: Invalid field definition
    Syn061,
    /// SYN080: Invalid expression
    Syn080,
    /// SYN081: Reserved operator used
    Syn081,
    /// SYN082: Invalid operator usage
    Syn082,
    /// SYN083: Missing semicolon (warning)
    Syn083,
    /// SYN090: Reserved syntax used
    Syn090,

    // === SEM: Semantic errors ===
    /// SEM010: Duplicate definition
    Sem010,
    /// SEM020: Undefined reference
    Sem020,
    /// SEM025: Unused import
    Sem025,
    /// SEM030: Invalid provider
    Sem030,
    /// SEM035: Missing required field
    Sem035,
    /// SEM040: Invalid model configuration
    Sem040,
    /// SEM050: Invalid prompt configuration
    Sem050,
    /// SEM060: Invalid tool definition
    Sem060,
    /// SEM061: Invalid agent definition
    Sem061,
    /// SEM070: Circular dependency
    Sem070,

    // === TYP: Type errors ===
    /// TYP001: Type mismatch
    Typ001,
    /// TYP002: Unknown type
    Typ002,
    /// TYP003: Invalid type expression
    Typ003,
    /// TYP010: Incompatible types in union
    Typ010,
    /// TYP030: Invalid bounded type
    Typ030,
    /// TYP031: Bound violation
    Typ031,
    /// TYP032: Invalid constraint
    Typ032,
    /// TYP040: Invalid generic type
    Typ040,

    // === CAP: Capability errors ===
    /// CAP001: Unknown capability
    Cap001,
    /// CAP002: Duplicate capability
    Cap002,
    /// CAP010: Capability not supported by provider
    Cap010,
    /// CAP020: Invalid capability configuration
    Cap020,

    // === PYB: Python bridge errors ===
    /// PYB001: Invalid Python bridge syntax
    Pyb001,
    /// PYB010: Python bridge runtime error
    Pyb010,

    // === RES: Resolution errors ===
    /// RES001: Unresolved name
    Res001,
}

impl ErrorCode {
    /// Returns the category prefix for this error code.
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Syn001
            | Self::Syn002
            | Self::Syn003
            | Self::Syn004
            | Self::Syn042
            | Self::Syn043
            | Self::Syn044
            | Self::Syn045
            | Self::Syn046
            | Self::Syn050
            | Self::Syn060
            | Self::Syn061
            | Self::Syn080
            | Self::Syn081
            | Self::Syn082
            | Self::Syn083
            | Self::Syn090 => "SYN",

            Self::Sem010
            | Self::Sem020
            | Self::Sem025
            | Self::Sem030
            | Self::Sem035
            | Self::Sem040
            | Self::Sem050
            | Self::Sem060
            | Self::Sem061
            | Self::Sem070 => "SEM",

            Self::Typ001
            | Self::Typ002
            | Self::Typ003
            | Self::Typ010
            | Self::Typ030
            | Self::Typ031
            | Self::Typ032
            | Self::Typ040 => "TYP",

            Self::Cap001 | Self::Cap002 | Self::Cap010 | Self::Cap020 => "CAP",

            Self::Pyb001 | Self::Pyb010 => "PYB",

            Self::Res001 => "RES",
        }
    }

    /// Returns the numeric portion of this error code.
    pub fn number(&self) -> u16 {
        match self {
            Self::Syn001 => 1,
            Self::Syn002 => 2,
            Self::Syn003 => 3,
            Self::Syn004 => 4,
            Self::Syn042 => 42,
            Self::Syn043 => 43,
            Self::Syn044 => 44,
            Self::Syn045 => 45,
            Self::Syn046 => 46,
            Self::Syn050 => 50,
            Self::Syn060 => 60,
            Self::Syn061 => 61,
            Self::Syn080 => 80,
            Self::Syn081 => 81,
            Self::Syn082 => 82,
            Self::Syn083 => 83,
            Self::Syn090 => 90,

            Self::Sem010 => 10,
            Self::Sem020 => 20,
            Self::Sem025 => 25,
            Self::Sem030 => 30,
            Self::Sem035 => 35,
            Self::Sem040 => 40,
            Self::Sem050 => 50,
            Self::Sem060 => 60,
            Self::Sem061 => 61,
            Self::Sem070 => 70,

            Self::Typ001 => 1,
            Self::Typ002 => 2,
            Self::Typ003 => 3,
            Self::Typ010 => 10,
            Self::Typ030 => 30,
            Self::Typ031 => 31,
            Self::Typ032 => 32,
            Self::Typ040 => 40,

            Self::Cap001 => 1,
            Self::Cap002 => 2,
            Self::Cap010 => 10,
            Self::Cap020 => 20,

            Self::Pyb001 => 1,
            Self::Pyb010 => 10,

            Self::Res001 => 1,
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{:03}", self.prefix(), self.number())
    }
}
