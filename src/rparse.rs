//! Functions and methods used to construct and compose parsers.
pub use c99_parsers::{identifier, decimal_number, octal_number, hex_number, float_number, char_literal, string_literal, comment, line_comment};
pub use parsers::{ParseStatus, ParseFailed, anycp, CharParsers, 
	match0, match1, match1_0, scan, seq2_ret_str, seq3_ret_str, seq4_ret_str, seq5_ret_str, StringParsers,
	fails, forward_ref, or_v, ret, seq2, seq3, seq4, seq5, seq6, seq7, seq8, seq9, seq2_ret0, seq2_ret1, seq3_ret0, seq3_ret1, seq3_ret2, seq4_ret0, 
	seq4_ret1, seq4_ret2, seq4_ret3, GenericParsers, Combinators, optional_str};
pub use misc::{EOT, is_alpha, is_digit, is_alphanum, is_print, is_whitespace};
pub use types::{Parser, State, Status, Succeeded, Failed};

// TODO: I think everything in the other modules needs to be marked priv

