// Test selected individual parse functions (the other tests suffice for most functions).
import io;
import io::writer_util;
import result::*;
import misc::*;
import test_helpers::*;
import types::*;

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
			log_err("unary", input, {old_state: input, err_state: {index: input.index with input}, mesg: "'-' or '+'"})
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
			log_err("digit", input, {old_state: input, err_state: {index: input.index with input}, mesg: "digit"})
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
fn test_thene()
{
	let p = parse_unary().thene({|c| parse_num(c)});
	
	assert check_int_ok("-9", p, -9);
	assert check_int_ok("+3", p, 3);
	assert check_int_failed("", p, "'-' or '+'", 1);
	assert check_int_failed("~9", p, "'-' or '+'", 1);
	assert check_int_failed("--9", p, "digit", 1);
	
	let text = chars_with_eot("~9");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;	// simple case where parse_unary fails
	
	let text = chars_with_eot("--");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;	// if parse_num fails we need to start over
}

#[test]
fn test_then()
{
	let p = "<".lit().then("foo".lit()).then(">".lit());
	
	assert check_str_ok("<foo>", p, ">");
	assert check_str_failed("", p, "Expected '<'", 1);
	assert check_str_failed("<", p, "Expected 'foo'", 1);
	assert check_str_failed("<foo", p, "Expected '>'", 1);
	assert check_str_failed("<foo-", p, "Expected '>'", 1);
	
	let text = chars_with_eot("<foo-");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;	// if any of the then clauses fails we need to start over
}


#[test]
fn test_seq()
{
	let prefix = match1(is_identifier_prefix, "Expected identifier");
	let suffix = match1(is_identifier_suffix, "Expected identifier").repeat0();
	let trailer = match1(is_identifier_trailer, "Expected identifier").optional("");
	let p = seq3(prefix, suffix, trailer, {|a, b, c| result::ok(a + str::connect(b, "") + c)});
	
	assert check_str_ok("hey", p, "hey");
	assert check_str_ok("hey?", p, "hey?");
	assert check_str_ok("hey!", p, "hey!");
	assert check_str_ok("hey_there", p, "hey_there");
	assert check_str_ok("hey there", p, "hey");
	assert check_str_ok("spanky123xy", p, "spanky123xy");
	assert check_str_failed("", p, "Expected identifier", 1);
	
	let p = seq2("a".lit(), "b".lit(), {|x, y| result::ok(x+y)});
	let text = chars_with_eot("az");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;
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
			log_err("lower", input, {old_state: input, err_state: {index: input.index with input}, mesg: "Expected lower-case letter"})
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
			log_err("upper", input, {old_state: input, err_state: {index: input.index with input}, mesg: "Expected upper-case letter"})
		}
	}
}

#[test]
fn test_or()
{
	let p = parse_lower().or(parse_upper());
	
	assert check_char_ok("a", p, 'a');
	assert check_char_ok("Z", p, 'Z');
	assert check_char_failed("", p, "Expected lower-case letter or upper-case letter", 1);
	assert check_char_failed("9", p, "Expected lower-case letter or upper-case letter", 1);
}

#[test]
fn test_or_v()
{
	let p = or_v(["a".lit(), "bb".lit(), "c".lit()]);
	
	assert check_str_ok("a", p, "a");
	assert check_str_ok("bb", p, "bb");
	assert check_str_ok("c", p, "c");
	assert check_str_ok("ca", p, "c");
	assert check_str_failed("", p, "Expected 'a' or 'bb' or 'c'", 1);
	
	let text = chars_with_eot("bz");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;
}
#[test]
fn test_optional()
{
	let p = "x".lit().optional("z");
	
	assert check_str_ok("x", p, "x");
	assert check_str_ok("b", p, "z");
	assert check_str_ok("", p, "z");
}

#[test]
fn test__repeat0()
{
	let p = "b".lit().repeat0();
	
	assert check_str_array_ok("", p, []);
	assert check_str_array_ok("b", p, ["b"]);
	assert check_str_array_ok("bb", p, ["b", "b"]);
	assert check_str_array_ok("bbb", p, ["b", "b", "b"]);
	assert check_str_array_ok("c", p, []);
}

#[test]
fn test__repeat1()
{
	let p = "b".lit().repeat1("b's");
	
	assert check_str_array_ok("b", p, ["b"]);
	assert check_str_array_ok("bb", p, ["b", "b"]);
	assert check_str_array_ok("bbb", p, ["b", "b", "b"]);
	
	assert check_str_array_failed("", p, "b's", 1);
	assert check_str_array_failed("c", p, "b's", 1);
}

#[test]
fn test_list()
{
	let p = "b".lit().list(",".lit());
	
	assert check_str_array_ok("b", p, ["b"]);
	assert check_str_array_ok("b,b", p, ["b", "b"]);
	assert check_str_array_ok("b,b,b", p, ["b", "b", "b"]);
	assert check_str_array_ok("b,b,c", p, ["b", "b"]);
	
	assert check_str_array_failed("", p, "Expected 'b'", 1);
	assert check_str_array_failed("c", p, "Expected 'b'", 1);
}


pure fn is_identifier_trailer(ch: char) -> bool
{
	ret ch == '?' || ch == '!';
}

#[test]
fn test_chainl1()
{
	let factor = integer();
	let op = "*".lit().or("/".lit());
	let p = factor.chainl1(op, {|lhs, op, rhs| if op == "*" {lhs * rhs} else {lhs / rhs}});
	
	assert check_int_ok("2", p, 2);
	assert check_int_ok("2*3", p, 6);
	assert check_int_ok("2*3/4", p, 1);
	assert check_int_ok("2*3/4/2", p, 0);
	assert check_int_ok("2*3-4", p, 6);
}

#[test]
fn test_chainr1()
{
	let factor = integer();
	let op = "*".lit().or("/".lit());
	let p = factor.chainr1(op, {|lhs, op, rhs| if op == "*" {lhs * rhs} else {lhs / rhs}});
	
	assert check_int_ok("2", p, 2);
	assert check_int_ok("2*3", p, 6);
	assert check_int_ok("2*3/4", p, 0);
	assert check_int_ok("2*3/4/2", p, 2);
	assert check_int_ok("2*3-4", p, 6);
}

#[test]
fn test_tag()
{
	let p = "<".lit().then("foo".lit()).then(">".lit()).tag("Expected bracketed foo");
	
	assert check_str_ok("<foo>", p, ">");
	assert check_str_failed("", p, "Expected bracketed foo", 1);
	assert check_str_failed("<", p, "Expected 'foo'", 1);
	assert check_str_failed("<foo", p, "Expected '>'", 1);
}

#[test]
fn test_parse()
{
	let p = "<".lit().s0().then("foo".lit().s0()).then(">".lit()).tag("Expected bracketed foo");
	
	alt parse(p, "unit test", "< foo\t>")
	{
		result::ok(s)
		{
			if s != ">"
			{
				io::stderr().write_line(#fmt["Expected '>' but found '%s'.", s]);
				assert false;
			}
		}
		result::err({file, line, col, mesg})
		{
			io::stderr().write_line(#fmt["Error '%s' on line %u and col %u.", mesg, line, col]);
			assert false;
		}
	}
	
	assert check_str_failed("<foo", p, "Expected '>'", 1);
	alt parse(p, "unit test", "< \n\nfoo\tx")
	{
		result::ok(s)
		{
			io::stderr().write_line(#fmt["Somehow parsed '%s'.", s]);
			assert false;
		}
		result::err({file, line, col, mesg})
		{
			assert file == "unit test";
			assert line == 3u;
			assert col == 5u;
			assert mesg == "Expected '>'";
		}
	}
}

