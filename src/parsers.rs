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
/// Parse functions which return a string.
trait StringParsers
{
	/// Returns s if input matches self. Also see liti and litv.
	fn lit() -> parser<@~str>;
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
