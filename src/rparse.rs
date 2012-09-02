//! Functions and methods used to construct and compose parsers.

use c99_parsers::*;
use char_parsers::*;
use combinators::*;
use generic_parsers::*;
use str_parsers::*;
use misc::*;
use parser::*;

// c99_parsers
export identifier, decimal_number, octal_number, hex_number, float_number, char_literal, string_literal, comment, line_comment;

// char parsers
export match, anyc, noc;

// combinators
export chainl1, chainr1, forward_ref, list, optional, or_v, r, r0, r1, seq2, seq3, seq4, seq5, seq6, seq7, seq8, seq9,
	seq2_ret0, seq2_ret1, seq3_ret0, seq3_ret1, seq3_ret2, seq4_ret0, seq4_ret1, seq4_ret2, seq4_ret3, s0, s1, then, thene;

// generic_parsers
export parser, state, status, succeeded, failed;

// generic_parsers
export litv, fails, return;

// misc
export EOT, is_alpha, is_digit, is_alphanum, is_print, is_whitespace;

// str_parsers
export liti, lit, match0, match1, match1_0, optional_str, scan, scan0, scan1, seq2_ret_str, seq3_ret_str, seq4_ret_str, seq5_ret_str;

// types
export parser, state, status, succeeded, failed;

// parser
export parse_status, parse_failed, eot, everything, parse, str_trait, str_methods, str_parser_trait, str_parser_methods, parser_trait, parser_methods;

