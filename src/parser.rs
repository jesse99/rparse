//! Parser traits and top-level parse functions.

use char_parsers::*;
use combinators::*;
use misc::*;
use types::*;

/// Return type of parse function.
type parse_status<T: copy owned> = result::Result<T, parse_failed>;

/// Returned by parse function on error. Line and col are both 1-based.
type parse_failed = {file: ~str, line: uint, col: uint, mesg: ~str};

/// Uses parser to parse text. Also see everything function.
fn parse<T: copy owned>(parser: parser<T>, file: ~str, text: &str) -> parse_status<T>
{
	let chars = chars_with_eot(text);
	let input = {file: file, text: chars, index: 0u, line: 1};
	match parser(input)
	{
		result::Ok(pass) =>
		{
			result::Ok(pass.value)
		}
		result::Err(failure) =>
		{
			let col = get_col(chars, failure.err_state.index);
			result::Err({file: failure.old_state.file, line: failure.err_state.line as uint, col: col, mesg: failure.mesg})
		}
	}
}

/// Returns a parser which matches the end of the input.
/// 
/// Typically clients will use the everything method instead of calling this directly.
fn eot() -> parser<()>
{
	{|input: state|
		if input.text[input.index] == EOT
		{
			result::Ok({new_state: {index: input.index + 1u ,.. input}, value: ()})
		}
		else
		{
			result::Err({old_state: input, err_state: input, mesg: ~"EOT"})
		}
	}
}

/// Parses the text and fails if all the text was not consumed. Leading space is allowed.
/// 
/// This is typically used in conjunction with the parse function. Note that space has to have the
/// same type as parser which is backwards from how it is normally used.
fn everything<T: copy owned, U: copy owned>(parser: parser<T>, space: parser<U>) -> parser<T>
{
	seq3_ret1(space, parser, eot())
}

/// Methods that treat a string as a literal.
trait str_trait
{
	fn lit() -> parser<~str>;
	fn liti() -> parser<~str>;
	fn litv<T: copy owned>(value: T) -> parser<T>;
	fn anyc() -> parser<char>;
	fn noc() -> parser<char>;
	fn s0() -> parser<~str>;
	fn s1() -> parser<~str>;
}

impl &str : str_trait
{
	fn lit() -> parser<~str>
	{
		lit(self)
	}
	
	fn liti() -> parser<~str>
	{
		liti(self)
	}
	
	fn litv<T: copy owned>(value: T) -> parser<T>
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
	
	fn s0() -> parser<~str>
	{
		s0(lit(self))
	}
	
	fn s1() -> parser<~str>
	{
		s1(lit(self))
	}
}

trait str_parser_trait
{
	fn optional_str() -> parser<~str>;
}

impl  parser<~str>: str_parser_trait 
{
	fn optional_str() -> parser<~str>
	{
		optional_str(self)
	}
}

/// These work the same as the functions of the same name, but tend
/// to make the code look a bit better.
trait parser_trait<T: copy owned>
{
	fn thene<U: copy owned>(eval: fn@ (T) -> parser<U>) -> parser<U>;
	fn then<U: copy owned>(parser2: parser<U>) -> parser<U>;
	fn or(parser2: parser<T>) -> parser<T>;
	fn optional() -> parser<Option<T>>;
	fn r(n: uint, m: uint) -> parser<~[T]>;
	fn r0() -> parser<~[T]>;
	fn r1() -> parser<~[T]>;
	fn list<U: copy owned>(sep: parser<U>) -> parser<~[T]>;
	fn chain_suffix<U: copy owned>(op: parser<U>) -> parser<~[(U, T)]>;
	fn chainl1<U: copy owned>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>;
	fn chainr1<U: copy owned>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>;
	
	fn note(mesg: ~str) -> parser<T>;
	fn err(label: &str) -> parser<T>;
	fn parse(file: ~str, text: ~str) -> parse_status<T>;
	
	fn s0() -> parser<T>;
	fn s1() -> parser<T>;
	fn everything<U: copy owned>(space: parser<U>) -> parser<T>;
}

impl <T: copy owned> parser<T> : parser_trait<T>
{
	fn thene<U: copy owned>(eval: fn@ (T) -> parser<U>) -> parser<U>
	{
		thene(self, eval)
	}
	
	fn then<U: copy owned>(parser2: parser<U>) -> parser<U>
	{
		then(self, parser2)
	}
	
	fn or(parser2: parser<T>) -> parser<T>
	{
		or(self, parser2)
	}
	
	fn optional() -> parser<Option<T>>
	{
		optional(self)
	}
	
	fn r(n: uint, m: uint) -> parser<~[T]>
	{
		r(self, n, m)
	}
	
	fn r0() -> parser<~[T]>
	{
		r0(self)
	}
	
	fn r1() -> parser<~[T]>
	{
		r1(self)
	}
	
	fn list<U: copy owned>(sep: parser<U>) -> parser<~[T]>
	{
		list(self, sep)
	}
	
	fn chain_suffix<U: copy owned>(op: parser<U>) -> parser<~[(U, T)]>
	{
		chain_suffix(self, op)
	}
	
	fn chainl1<U: copy owned>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
	{
		chainl1(self, op, eval)
	}
	
	fn chainr1<U: copy owned>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
	{
		chainr1(self, op, eval)
	}
	
	fn parse(file: ~str, text: ~str) -> parse_status<T>
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
	
	fn everything<U: copy owned>(space: parser<U>) -> parser<T>
	{
		everything(self, space)
	}
}



