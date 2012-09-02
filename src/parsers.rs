//! Functions and methods that return functions that are able to parse strings.
//!
//! These can be divided into parsers that return chars, strings, and generic Ts.
// TODO: probably should use individual modules for these, but the dependencies
// are painful (see https://github.com/mozilla/rust/issues/3352).
use misc::*;
use types::{parser, state, status};

// ---- char parsers ------------------------------------------------------------------------------
/// Consumes a character which must satisfy the predicate.
/// Returns the matched character.
fn anycp(predicate: fn@ (char) -> bool) -> parser<char>
{
	|input: state| {
		let mut i = input.index;
		if input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		if i > input.index
		{
			result::Ok({new_state: {index: i ,.. input}, value: input.text[input.index]})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: i ,.. input}, mesg: @~""})
		}
	}
}

/// Parse functions which return a character.
trait CharParsers
{
	/// Attempts to match any character in self. If matched the char is returned.
	fn anyc() -> parser<char>;
	
	/// Attempts to match no character in self. If matched the char is returned.
	fn noc() -> parser<char>;
}

impl &str : CharParsers
{
	fn anyc() -> parser<char>
	{
		// Note that we're handing this string off to a closure so we can't get rid of this copy
		// even if we make the impl on ~str.
		let s = self.to_unique();
		
		|input: state|
		{
			let mut i = input.index;
			if str::find_char(s, input.text[i]).is_some()
			{
				i += 1u;
			}
			
			if i > input.index
			{
				result::Ok({new_state: {index: i ,.. input}, value: input.text[input.index]})
			}
			else
			{
				result::Err({old_state: input, err_state: {index: i ,.. input}, mesg: @fmt!("[%s]", s)})
			}
		}
	}
	
	fn noc() -> parser<char>
	{
		let s = self.to_unique();
		
		|input: state|
		{
			let mut i = input.index;
			if input.text[i] != EOT && str::find_char(s, input.text[i]).is_none()
			{
				i += 1u;
			}
			
			if i > input.index
			{
				result::Ok({new_state: {index: i ,.. input}, value: input.text[input.index]})
			}
			else
			{
				result::Err({old_state: input, err_state: {index: i ,.. input}, mesg: @fmt!("[^%s]", s)})
			}
		}
	}
}

// ---- string parsers ----------------------------------------------------------------------------
// It would be a lot more elegant if match0, match1, and co were removed
// and users relied on composition to build the sort of parsers that they
// want. However in practice this is not such a good idea:
// 1) Matching is a very common operation, but instead of something simple
// like:
//    match0(p)
// users would have to write something like:
//    match(p).r0().str()
// 2) Generating an array of characters and then converting them into a string
// is much slower than updating a mutable string.
// 3) Debugging a parser is simpler if users can use higher level building
// blocks (TODO: though maybe we can somehow ignore or collapse low
// level parsers when logging).

/// Consumes zero or more characters matching the predicate.
/// Returns the matched characters. 
/// 
/// Note that this does not increment line.
fn match0(predicate: fn@ (char) -> bool) -> parser<@~str>
{
	|input: state|
	{
		let mut i = input.index;
		while input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		let text = str::from_chars(vec::slice(input.text, input.index, i));
		result::Ok({new_state: {index: i, ..input}, value: @text})
	}
}

/// Consumes one or more characters matching the predicate.
/// Returns the matched characters. 
/// 
/// Note that this does not increment line.
fn match1(predicate: fn@ (char) -> bool) -> parser<@~str>
{
	|input: state|
	{
		let mut i = input.index;
		while input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		if i > input.index
		{
			let text = str::from_chars(vec::slice(input.text, input.index, i));
			result::Ok({new_state: {index: i, ..input}, value: @text})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: i, ..input}, mesg: @~""})
		}
	}
}

/// match1_0 := prefix+ suffix*
fn match1_0(prefix: fn@ (char) -> bool, suffix: fn@ (char) -> bool) -> parser<@~str>
{
	let prefix = match1(prefix);
	let suffix = match0(suffix);
	prefix.thene(|p| suffix.thene(|s| ret(@(*p + *s))))
}

/// Parse functions which return a string.
trait StringParsers
{
	/// Returns the input that matches self. Also see liti and litv.
	fn lit() -> parser<@~str>;
	
	/// Returns the input that matches lower-cased self. Also see lit and litv.
	fn liti() -> parser<@~str>;
}

impl &str : StringParsers
{
	fn lit() -> parser<@~str>
	{
		let s = self.to_unique();
		
		|input: state|
		{
			let mut i = 0u;
			let mut j = input.index;
			while i < str::len(s)
			{
				let {ch, next} = str::char_range_at(s, i);
				if ch == input.text[j]
				{
					i = next;
					j += 1u;
				}
				else
				{
					break;
				}
			}
			
			if i == str::len(s)
			{
				let text = str::from_chars(vec::slice(input.text, input.index, j));
				result::Ok({new_state: {index: j ,.. input}, value: @text})
			}
			else
			{
				result::Err({old_state: input, err_state: {index: j ,.. input}, mesg: @fmt!("'%s'", s)})
			}
		}
	}
	
	fn liti() -> parser<@~str>
	{
		let s = str::to_lower(self);
		
		|input: state|
		{
			let mut i = 0u;
			let mut j = input.index;
			while i < str::len(s)
			{
				let {ch, next} = str::char_range_at(s, i);
				if ch == lower_char(input.text[j])
				{
					i = next;
					j += 1u;
				}
				else
				{
					break;
				}
			}
			
			if i == str::len(s)
			{
				let text = str::from_chars(vec::slice(input.text, input.index, j));
				result::Ok({new_state: {index: j ,.. input}, value: @text})
			}
			else
			{
				result::Err({old_state: input, err_state: {index: j ,.. input}, mesg: @fmt!("'%s'", s)})
			}
		}
	}
}

// ---- generic parsers ---------------------------------------------------------------------------
/// Returns a parser which always fails.
fn fails<T: copy owned>(mesg: &str) -> parser<T>
{
	let mesg = mesg.to_unique();
	|input: state| result::Err({old_state: input, err_state: input, mesg: @copy mesg})
}

/// Returns a parser which always succeeds, but does not consume any input.
fn ret<T: copy owned>(value: T) -> parser<T>
{
	|input: state| result::Ok({new_state: input, value: value})
}

/// Parse functions which return a generic type.
trait GenericParsers
{
	/// Returns value if input matches s. Also see lit.
	fn litv<T: copy owned>(value: T) -> parser<T>;
}

trait Combinators<T: copy owned>
{
	/// If parser is successful then the function returned by eval is called
	/// with parser's result. If parser fails eval is not called.
	/// 
	/// Often used to translate parsed values: `p().thene({|pvalue| return(2*pvalue)})`
	fn thene<U: copy owned>(eval: fn@ (T) -> parser<U>) -> parser<U>;
}

impl<T: copy owned> parser<T> : Combinators<T>
{
	fn thene<U: copy owned>(eval: fn@ (T) -> parser<U>) -> parser<U>
	{
		|input: state|
		{
			do result::chain(self(input))
			|pass|
			{
				do result::chain_err(eval(pass.value)(pass.new_state))
					|failure| {result::Err({old_state: input, ..failure})}
			}
		}
	}
}

impl &str : GenericParsers
{
	fn litv<T: copy owned>(value: T) -> parser<T>
	{
		let s = self.to_unique();
		
		|input: state|
		{
			match s.lit()(input)
			{
				result::Ok(pass) =>
				{
					result::Ok({new_state: pass.new_state, value: value})
				}
				result::Err(failure) =>
				{
					result::Err(failure)
				}
			}
		}
	}
}
