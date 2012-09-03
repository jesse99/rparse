//! Parser traits and top-level parse functions.

//use char_parsers::*;
//use combinators::*;
//use misc::*;
//use types::*;

/// Return type of parse function.
type parse_status<T: copy owned> = result::Result<T, parse_failed>;

/// Returned by parse function on error. Line and col are both 1-based.
type parse_failed = {file: ~str, line: uint, col: uint, mesg: ~str};

/// Uses parser to parse text. Also see everything function.
fn parse<T: copy owned>(parser: Parser<T>, file: ~str, text: &str) -> parse_status<T>
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

