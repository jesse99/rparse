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
			result::ok({new_state: {index: input.index + 1u with input}, value: ()})
		}
		else
		{
			result::err({old_state: input, err_state: input, mesg: "EOT"})
		}
	}
}

#[doc = "Parses the text and fails if all the text was not consumed. Leading space is allowed.

This is typically used in conjunction with the parse function. Note that space has to have the
same type as parser which is backwards from how it is normally used."]
fn everything<T: copy, U: copy>(parser: parser<T>, space: parser<U>) -> parser<T>
{
	seq3_ret1(space, parser, eot())
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
	
	fn r(n: uint, m: uint) -> parser<[T]/~>
	{
		r(self, n, m)
	}
	
	fn r0() -> parser<[T]/~>
	{
		r0(self)
	}
	
	fn r1() -> parser<[T]/~>
	{
		r1(self)
	}
	
	fn list<U: copy>(sep: parser<U>) -> parser<[T]/~>
	{
		list(self, sep)
	}
	
	fn chain_suffix<U: copy>(op: parser<U>) -> parser<[(U, T)]/~>
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
	
	#[doc = "Logs the result of the previous parser.
	
	If it was successful then the log is at INFO level. Otherwise it is at DEBUG level."]
	fn note(mesg: str) -> parser<T>
	{
		{|input: state|
			alt self(input)
			{
				result::ok(pass)
				{
					// Note that we make multiple calls to munge_chars which is fairly slow, but
					// we only do that when actually logging: when info or debug logging is off
					// the munge_chars calls aren't evaluated.
					assert pass.new_state.index >= input.index;			// can't go backwards on success (but no progress is fine, eg e*)
					if pass.new_state.index > input.index
					{
						#info("%s", munge_chars(input.text));
						#info("%s^ %s parsed '%s'", repeat_char(' ', pass.new_state.index), mesg, str::slice(munge_chars(input.text), input.index, pass.new_state.index));
					}
					else
					{
						#info("%s", munge_chars(input.text));
						#info("%s^ %s passed", repeat_char(' ', pass.new_state.index), mesg);
					}
					result::ok(pass)
				}
				result::err(failure)
				{
					assert failure.old_state.index == input.index;			// on errors the next parser must begin at the start
					assert failure.err_state.index >= input.index;			// errors can't be before the input
					
					#debug("%s", munge_chars(input.text));
					if failure.err_state.index > input.index 
					{
						#debug("%s^%s! %s failed", repeat_char('-', input.index), repeat_char(' ', failure.err_state.index - input.index), mesg);
					}
					else
					{
						#debug("%s^ %s failed", repeat_char('-', input.index), mesg);
					}
					result::err(failure)
				}
			}
		}
	}
	
	#[doc = "Like note except that the mesg is also used for error reporting.
	
	If label is not empty then it is used if the previous parser completely failed to parse or if its error
	message was empty. Otherwise it suppresses errors from the parser (in favor of a later err function).
	Non-empty labels should look like \"expression\" or \"statement\"."]
	fn err(label: str) -> parser<T>
	{
		{|input: state|
			result::chain_err((self.note(label))(input))
			{|failure|
				if str::is_empty(label)
				{
					result::err({mesg: "" with failure})
				}
				else if failure.err_state.index == input.index || str::is_empty(failure.mesg)
				{
					result::err({mesg: label with failure})
				}
				else
				{
					// If we managed to parse something then it is usually better to
					// use that error message. (If that's not what you want then use
					// empty strings there).
					result::err(failure)
				}
			}
		}
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
	
	fn everything<U: copy>(space: parser<U>) -> parser<T>
	{
		everything(self, space)
	}
}



