//! Parser functions with char return types.
use misc::*;
use types::{parser, state, status};

/// Consumes a character which must satisfy the predicate.
/// Returns the matched character.
fn anyp(predicate: fn@ (char) -> bool) -> parser<char>
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
			result::Err({old_state: input, err_state: {index: i ,.. input}, mesg: ~""})
		}
	}
}

/// Attempts to match any character in chars. If matched the char is returned.
fn anyc(chars: &str) -> parser<char>
{
	let chars = unslice(chars);
	
	|input: state| {
		let mut i = input.index;
		if str::find_char(chars, input.text[i]).is_some()
		{
			i += 1u;
		}
		
		if i > input.index
		{
			result::Ok({new_state: {index: i ,.. input}, value: input.text[input.index]})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: i ,.. input}, mesg: fmt!("[%s)", unslice(chars))})
		}
	}
}

/// Attempts to match no character in chars. If matched the char is returned.
fn noc(chars: &str) -> parser<char>
{
	let chars = unslice(chars);
	
	|input: state| {
		let mut i = input.index;
		if input.text[i] != EOT && str::find_char(chars, input.text[i]).is_none()
		{
			i += 1u;
		}
		
		if i > input.index
		{
			result::Ok({new_state: {index: i ,.. input}, value: input.text[input.index]})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: i ,.. input}, mesg: fmt!("[^%s)", unslice(chars))})
		}
	}
}

