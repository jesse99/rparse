use misc::*;
use parsers::*;
use test_helpers::*;

// Usually these would be written using then, but we are using this
// to test then and don't want to confuse things by testing then 
// multiple times for each input string.
fn parse_unary() -> Parser<char>
{
	|input: State|
	{
		let ch = input.text[input.index];
		if ch == '-' || ch == '+'
		{
			result::Ok({new_state: {index: input.index + 1u ,.. input}, value: ch})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: input.index ,.. input}, mesg: @~"'-' or '+'"})
		}
	}
}

fn parse_digit() -> Parser<int>
{
	|input: State|
	{
		let ch = input.text[input.index];
		if ch >= '0' && ch <= '9'
		{
			let value = option::get(char::to_digit(ch, 10u)) as int;
			result::Ok({new_state: {index: input.index + 1u ,.. input}, value: value})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: input.index ,.. input}, mesg: @~"digit"})
		}
	}
}

fn parse_num(op: char) -> Parser<int>
{
	|input: State|
	{
		do result::chain(parse_digit()(input))
		|output|
		{
			let value = if op == '-' {-output.value} else {output.value};
			result::Ok({value: value, ..output})
		}
	}
}

#[test]
fn test_fails()
{
	let p = fails::<char>("ack");
	
	assert check_char_failed("", p, "ack", 1);
	assert check_char_failed("9", p, "ack", 1);
}

#[test]
fn test_ret()
{
	let p = ret('x');
	
	assert check_char_ok("a", p, 'x');
	assert check_char_ok("e", p, 'x');
	assert check_char_ok(" ", p, 'x');
}

#[test]
fn test_litv()
{
	let p = "foo".litv(@~"hmm");
	
	assert check_str_ok("foo", p, "hmm");
	assert check_str_ok("foo-shizzle", p, "hmm");
	assert check_str_failed("", p, "'foo'", 1);
	assert check_str_failed("bar", p, "'foo'", 1);
	assert check_str_failed("pseudo foo", p, "'foo'", 1);
}

#[test]
fn test_seq3()
{
	let p = do seq3("+-".anyc(), anycp(is_digit), anycp(is_digit))
		|a, b, c|
		{
			let x = (10*char::to_digit(b, 10).get() + char::to_digit(c, 10).get()) as int; 
			result::Ok(if a == '-' {-x} else {x})
		};
	
	assert check_int_ok("+23", p, 23);
	assert check_int_ok("+239", p, 23);
	assert check_int_ok("-19", p, -19);
	assert check_int_failed("", p, "[+-]", 1);
	assert check_int_failed("+2", p, "", 1);
	assert check_int_failed("2", p, "[+-]", 1);
}

#[test]
fn test_then()
{
	let p = "<".lit().then("foo".lit()).then(">".lit());
	
	assert check_str_ok("<foo>", p, ">");
	assert check_str_failed("", p, "'<'", 1);
	assert check_str_failed("<", p, "'foo'", 1);
	assert check_str_failed("<foo", p, "'>'", 1);
	assert check_str_failed("<foo-", p, "'>'", 1);
	
	let text = chars_with_eot("<foo-");
	let result = p({file: @~"unit test", text: text, index: 0u, line: 1});
	assert result::get_err(result).old_state.index == 0u;	// if any of the then clauses fails we need to start over
}

#[test]
fn test_thene()
{
	let p = do parse_unary().thene |c| {parse_num(c)};
	
	assert check_int_ok("-9", p, -9);
	assert check_int_ok("+3", p, 3);
	assert check_int_failed("", p, "'-' or '+'", 1);
	assert check_int_failed("~9", p, "'-' or '+'", 1);
	assert check_int_failed("--9", p, "digit", 1);
	
	let text = chars_with_eot("~9");
	let result = p({file: @~"unit test", text: text, index: 0u, line: 1});
	assert result::get_err(result).old_state.index == 0u;	// simple case where parse_unary fails
	
	let text = chars_with_eot("--");
	let result = p({file: @~"unit test", text: text, index: 0u, line: 1});
	assert result::get_err(result).old_state.index == 0u;	// if parse_num fails we need to start over
}
