#[doc = "Functions that can be used to parse C99 lexical elements (or with languages
that have similar lexical elements)."];
//import misc::*;
//import types::*;

// See http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1539.pdf
export identifier, integer;

pure fn is_identifier_prefix(ch: char) -> bool
{
	ret is_alpha(ch) || ch == '_';
}

pure fn is_identifier_suffix(ch: char) -> bool
{
	ret is_identifier_prefix(ch) || is_digit(ch);
}

#[doc = "identifier := [a-zA-Z_] [a-zA-Z0-9_]*

Note that match1_0 can be used to easily implement custom identifier parsers."]
fn identifier() -> parser<str>
{
	// Supposed to support universal character names too, e.g.
	// fo\u006F is a valid C99 identifier.
	match1_0(is_identifier_prefix, is_identifier_suffix, "Expected identifier")
}

#[doc = "integer := [+-]? [0-9]+"]
fn integer() -> parser<int>
{
	let digits = match1(is_digit, "Expected digits").thene({|s| return(option::get(int::from_str(s)))});
	let case1 = lit("+").then(digits);
	let case2 = seq2(lit("-"), digits, {|_o, v| result::ok(-v)});
	let case3 = digits;
	or_v([case1, case2, case3])
}

// replace match0(p) with match(p).r0()?
// invert
// match_any and match_none
// add a comment about inverse matching and EOT

// add (or move) unit tests for identifier and integer
// integer unit tests will probably need to be revised
// do we even need unit tests? maybe for floats?

// decimal_number		non-zero-digit	digit*
// octal_number			0	[0-7]+
// hex_number			0[xX]	[0-9a-fA-F]+
// integer_literal			decimal | octal | hex (u|U|l|L|ll|LL)?

// exponent := [eE] [+-]? digit-sequence
// float_number := [0-9]+? '.' [0-9]+ exponent?
// float_number := [0-9]+ '.' exponent?
// float_number := [0-9]+ exponent

// float_literal			float_number  [flFL]?
//    skipped hex float constants
// char_literal
// string_literal
// comment				these don't nest
// line_comment
