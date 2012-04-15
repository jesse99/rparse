//import io;
//import io::writer_util;
//import result = result::result;
import basis::*;
import misc::*;
import test_helpers::*;

#[test]
fn test_fails()
{
	let p = fails("oops");
	
	assert check_int_failed("", p, "oops", 1);
	assert check_int_failed("1", p, "oops", 1);
	assert check_int_failed("hello", p, "oops", 1);
}

#[test]
fn test_return()
{
	let p = return(42);
	
	assert check_int_ok("", p, 42);
	assert check_int_ok("1", p, 42);
	assert check_int_ok("22", p, 42);
}

#[test]
fn test_next()
{
	let p = next();
	
	assert check_char_ok("x", p, 'x');
	assert check_char_failed("", p, "EOT", 1);
}

// Usually these would be written using then, but we are using this
// to test then and don't want to confuse things by testing then 
// multiple times for each input string.
fn parse_unary() -> parser<char>
{
	{|input: state|
		let ch = input.text[input.index];
		if ch == '-' || ch == '+'
		{
			log_ok("unary", input, {new_state: {index: input.index + 1u with input}, value: ch})
		}
		else
		{
			log_err("unary", input, {new_state: input, max_index: input.index, mesg: "'-' or '+'"})
		}
	}
}

fn parse_digit() -> parser<int>
{
	{|input: state|
		let ch = input.text[input.index];
		if ch >= '0' && ch <= '9'
		{
			let value = option::get(char::to_digit(ch, 10u)) as int;
			log_ok("digit", input, {new_state: {index: input.index + 1u with input}, value: value})
		}
		else
		{
			log_err("digit", input, {new_state: input, max_index: input.index, mesg: "digit"})
		}
	}
}

fn parse_num(op: char) -> parser<int>
{
	{|input: state|
		chain(parse_digit()(input))
		{|output|
			let value = if op == '-' {-output.value} else {output.value};
			log_ok("num", input, {value: value with output})
		}
	}
}

#[test]
fn test_then()
{
	let p = parse_unary().then({|c| parse_num(c)});
	
	assert check_int_ok("-9", p, -9);
	assert check_int_ok("+3", p, 3);
	assert check_int_failed("", p, "'-' or '+'", 1);
	assert check_int_failed("~9", p, "'-' or '+'", 1);
	assert check_int_failed("--9", p, "digit", 1);
	
	let text = chars_with_eot("~9");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).new_state.index == 0u;	// simple case where parse_unary fails
	
	let text = chars_with_eot("--");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).new_state.index == 0u;	// if parse_num fails we need to start over
}

fn parse_lower() -> parser<char>
{
	{|input: state|
		let ch = input.text[input.index];
		if ch >= 'a' && ch <= 'z'
		{
			log_ok("lower", input, {new_state: {index: input.index + 1u with input}, value: ch})
		}
		else
		{
			log_err("lower", input, {new_state: input, max_index: input.index, mesg: "lower-case letter"})
		}
	}
}

fn parse_upper() -> parser<char>
{
	{|input: state|
		let ch = input.text[input.index];
		if ch >= 'A' && ch <= 'Z'
		{
			log_ok("upper", input, {new_state: {index: input.index + 1u with input}, value: ch})
		}
		else
		{
			log_err("upper", input, {new_state: input, max_index: input.index, mesg: "upper-case letter"})
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
