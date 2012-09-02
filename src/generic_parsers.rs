//! Parser functions with generic return types.
use misc::*;
use str_parsers::*;
use types::*;

/// Returns value if input matches s. Also see lit.
fn litv<T: copy owned>(s: &str, value: T) -> parser<T>
{
	let s = unslice(s);
	
	{|input: state|
		match lit(s)(input)
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

/// Returns a parser which always fails.
fn fails<T: copy owned>(mesg: &str) -> parser<T>
{
	let mesg = unslice(mesg);
	
	{|input: state|
		result::Err({old_state: input, err_state: input, mesg: unslice(mesg)})
	}
}

/// Returns a parser which always succeeds, but does not consume any input.
fn ret<T: copy owned>(value: T) -> parser<T>
{
	{|input: state|
		result::Ok({new_state: input, value: value})
	}
}

