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

#[test]
fn test_seq()
{
	let prefix = match1(is_identifier_prefix);
	let suffix = match1(is_identifier_suffix).r0();
	let trailer = match1(is_identifier_trailer).optional();
	let p = seq3(prefix, suffix, trailer, |a, b, c| result::Ok(a + str::connect(b, ~"") + option::get_default(c, ~"")) ).err("identifier");
	
	assert check_str_ok("hey", p, "hey");
	assert check_str_ok("hey?", p, "hey?");
	assert check_str_ok("hey!", p, "hey!");
	assert check_str_ok("hey_there", p, "hey_there");
	assert check_str_ok("hey there", p, "hey");
	assert check_str_ok("spanky123xy", p, "spanky123xy");
	assert check_str_failed("", p, "identifier", 1);
	
	let p = seq2("a".lit(), "b".lit(), |x, y| result::Ok(x+y) );
	let text = chars_with_eot("az");
	let result = p({file: ~"unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;
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

#[test]
fn test_or()
{
	let p = parse_lower().or(parse_upper());
	
	assert check_char_ok("a", p, 'a');
	assert check_char_ok("Z", p, 'Z');
	assert check_char_failed("", p, "lower-case letter or upper-case letter", 1);
	assert check_char_failed("9", p, "lower-case letter or upper-case letter", 1);
}

pure fn is_identifier_trailer(ch: char) -> bool
{
	return ch == '?' || ch == '!';
}

#[test]
fn test_tag()
{
	let p = "<".lit().then("foo".lit()).then(">".lit()).err("bracketed foo");
	
	assert check_str_ok("<foo>", p, ">");
	assert check_str_failed("", p, "bracketed foo", 1);
	assert check_str_failed("<", p, "'foo'", 1);
	assert check_str_failed("<foo", p, "'>'", 1);
}

#[test]
fn test_parse()
{
	let p = "<".lit().s0().then("foo".lit().s0()).then(">".lit()).err("bracketed foo");
	
	match parse(p, ~"unit test", ~"< foo\t>")
	{
		result::Ok(s) =>
		{
			if s != ~">"
			{
				io::stderr().write_line(fmt!("'>' but found '%s'.", s));
				assert false;
			}
		}
		result::Err({file, line, col, mesg}) =>
		{
			io::stderr().write_line(fmt!("Error '%s' on line %u and col %u.", mesg, line, col));
			assert false;
		}
	}
	
	assert check_str_failed("<foo", p, "'>'", 1);
	match parse(p, ~"unit test", ~"< \n\nfoo\tx")
	{
		result::Ok(s) =>
		{
			io::stderr().write_line(fmt!("Somehow parsed '%s'.", s));
			assert false;
		}
		result::Err({file, line, col, mesg}) =>
		{
			assert file == ~"unit test";
			assert line == 3u;
			assert col == 5u;
			assert mesg == ~"'>'";
		}
	}
}

