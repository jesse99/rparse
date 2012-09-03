// Test selected individual parse functions (the other tests suffice for most functions).
//use io;
use io::WriterUtil;
use c99_parsers::*;
use combinators::*;
use generic_parsers::*;
use misc::*;
use result::*;
use parser::*;
use test_helpers::*;

pure fn is_identifier_prefix(ch: char) -> bool
{
	return is_alpha(ch) || ch == '_';
}

pure fn is_identifier_suffix(ch: char) -> bool
{
	return is_identifier_prefix(ch) || is_digit(ch);
}

fn parse_lower() -> parser<char>
{
	|input: state| {
		let ch = input.text[input.index];
		if ch >= 'a' && ch <= 'z'
		{
			result::Ok({new_state: {index: input.index + 1u ,.. input}, value: ch})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: input.index ,.. input}, mesg: ~"lower-case letter"})
		}
	}
}

fn parse_upper() -> parser<char>
{
	|input: state| {
		let ch = input.text[input.index];
		if ch >= 'A' && ch <= 'Z'
		{
			result::Ok({new_state: {index: input.index + 1u ,.. input}, value: ch})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: input.index ,.. input}, mesg: ~"upper-case letter"})
		}
	}
}

pure fn is_identifier_trailer(ch: char) -> bool
{
	return ch == '?' || ch == '!';
}

