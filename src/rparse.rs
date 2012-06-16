#[doc = "Functions and methods used to construct and compose parsers."];

import c99_parsers::*;
import char_parsers::*;
import combinators::*;
import generic_parsers::*;
import str_parsers::*;
import misc::*;

// c99_parsers
export identifier, decimal_number, octal_number, hex_number, float_number, char_literal, string_literal, comment, line_comment;

// char parsers
export match, anyc, noc;

// combinators
export chainl1, chainr1, forward_ref, list, optional, or_v, r, r0, r1, seq2, seq3, seq4, seq5, seq6, seq7, seq8, seq9,
	seq2_ret0, seq2_ret1, seq3_ret0, seq3_ret1, seq3_ret2, seq4_ret0, seq4_ret1, seq4_ret2, seq4_ret3, s0, s1, tag, then, thene;

// generic_parsers
export parser, state, status, succeeded, failed;

// generic_parsers
export litv, fails, return;

// misc
export log_ok, log_err, EOT, is_alpha, is_digit, is_alphanum, is_print, is_whitespace;

// str_parsers
export liti, lit, match0, match1, match1_0, optional_str, scan, scan0, scan1, seq2_ret_str, seq3_ret_str, seq4_ret_str, seq5_ret_str;

// types
export parser, state, status, succeeded, failed;

export parse_status, parse_failed, eot, everything, parse, str_methods, parser_methods;


#[doc = "Return type of parse function."]
type parse_status<T: copy> = result::result<T, parse_failed>;

#[doc = "Returned by parse function on error. Line and col are both 1-based."]
type parse_failed = {file: str, line: uint, col: uint, mesg: str};

#[doc = "Uses parser to parse text. Also see everything function."]
fn parse<T: copy>(parser: parser<T>, file: str, text: str) -> parse_status<T>
{
	let chars = chars_with_eot(text);
	let input = {file: file, text: chars, index: 0u, line: 1};
	alt parser(input)
	{
		result::ok(pass)
		{
			result::ok(pass.value)
		}
		result::err(failure)
		{
			let col = get_col(chars, failure.err_state.index);
			result::err({file: failure.old_state.file, line: failure.err_state.line as uint, col: col, mesg: failure.mesg})
		}
	}
}

#[doc = "Returns a parser which matches the end of the input.

Typically clients will use the everything method instead of calling this directly."]
fn eot() -> parser<()>
{
	{|input: state|
		if input.text[input.index] == EOT
		{
			log_ok("eot", input, {new_state: {index: input.index + 1u with input}, value: ()})
		}
		else
		{
			log_err("eot", input, {old_state: input, err_state: input, mesg: "Expected EOT"})
		}
	}
}

#[doc = "Parses the text and fails if all the text was not consumed. Leading space is allowed.

This is typically used in conjunction with the parse function. Note that space has to have the
same type as parser which is backwards from how it is normally used. To get this to work you
can use a syntax like: `return(x).s0()` where x is of type T."]
fn everything<T: copy>(parser: parser<T>, space: parser<T>) -> parser<T>
{
	seq3(space, parser, eot()) {|_a, b, _c| result::ok(b)}
}

#[doc = "Methods that treat a string as a literal."]
impl str_methods for str
{
	fn lit() -> parser<str>
	{
		lit(self)
	}
	
	fn liti() -> parser<str>
	{
		liti(self)
	}
	
	fn litv<T: copy>(value: T) -> parser<T>
	{
		litv(self, value)
	}
	
	fn anyc() -> parser<char>
	{
		anyc(self)
	}
	
	fn noc() -> parser<char>
	{
		noc(self)
	}
	
	fn s0() -> parser<str>
	{
		s0(lit(self))
	}
	
	fn s1() -> parser<str>
	{
		s1(lit(self))
	}
}

impl str_parser_methods for parser<str>
{
	fn optional_str() -> parser<str>
	{
		optional_str(self)
	}
}

#[doc = "These work the same as the functions of the same name, but tend
to make the code look a bit better."]
impl parser_methods<T: copy> for parser<T>
{
	fn thene<U: copy>(eval: fn@ (T) -> parser<U>) -> parser<U>
	{
		thene(self, eval)
	}
	
	fn then<U: copy>(parser2: parser<U>) -> parser<U>
	{
		then(self, parser2)
	}
	
	fn or(parser2: parser<T>) -> parser<T>
	{
		or(self, parser2)
	}
	
	fn optional() -> parser<option<T>>
	{
		optional(self)
	}
	
	fn r(n: uint, m: uint) -> parser<[T]>
	{
		r(self, n, m)
	}
	
	fn r0() -> parser<[T]>
	{
		r0(self)
	}
	
	fn r1() -> parser<[T]>
	{
		r1(self)
	}
	
	fn list<U: copy>(sep: parser<U>) -> parser<[T]>
	{
		list(self, sep)
	}
	
	fn chain_suffix<U: copy>(op: parser<U>) -> parser<[(U, T)]>
	{
		chain_suffix(self, op)
	}
	
	fn chainl1<U: copy>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
	{
		chainl1(self, op, eval)
	}
	
	fn chainr1<U: copy>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
	{
		chainr1(self, op, eval)
	}
	
	fn tag(label: str) -> parser<T>
	{
		tag(self, label)
	}
	
	fn parse(file: str, text: str) -> parse_status<T>
	{
		parse(self, file, text)
	}
	
	// ---------------------------------------------------------------------------
	fn s0() -> parser<T>
	{
		s0(self)
	}
	
	fn s1() -> parser<T>
	{
		s1(self)
	}
	
	fn everything(space: parser<T>) -> parser<T>
	{
		everything(self, space)
	}
}



