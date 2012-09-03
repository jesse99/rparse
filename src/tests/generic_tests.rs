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
fn test_chainl1()
{
	let factor = parse_digit();
	let op = "*".lit().or("/".lit());
	let p = factor.chainl1(op, |lhs, op, rhs| if op == @~"*" {lhs * rhs} else {lhs / rhs} );
	
	assert check_int_ok("2", p, 2);
	assert check_int_ok("2*3", p, 6);
	assert check_int_ok("2*3/4", p, 1);
	assert check_int_ok("2*3/4/2", p, 0);
	assert check_int_ok("2*3-4", p, 6);
}

#[test]
fn test_chainr1()
{
	let factor = parse_digit();
	let op = "*".lit().or("/".lit());
	let p = factor.chainr1(op, |lhs, op, rhs| if op == @~"*" {lhs * rhs} else {lhs / rhs} );
	
	assert check_int_ok("2", p, 2);
	assert check_int_ok("2*3", p, 6);
	assert check_int_ok("2*3/4", p, 0);
	assert check_int_ok("2*3/4/2", p, 2);
	assert check_int_ok("2*3-4", p, 6);
}

#[test]
fn test_err()
{
	let p = "<".lit().then("foo".lit()).then(">".lit()).err("bracketed foo");
	
	assert check_str_ok("<foo>", p, ">");
	assert check_str_failed("", p, "bracketed foo", 1);
	assert check_str_failed("<", p, "'foo'", 1);
	assert check_str_failed("<foo", p, "'>'", 1);
}

#[test]
fn test_everything()
{
	let s = ret(0).s0();
	let p = parse_digit().everything(s);
	
	assert check_int_ok("2", p, 2);
	assert check_int_ok("   \t3", p, 3);
	assert check_int_failed("2 ", p, "EOT", 1);
	assert check_int_failed("\t2\n", p, "EOT", 1);
}

#[test]
fn test_fails()
{
	let p = fails::<char>("ack");
	
	assert check_char_failed("", p, "ack", 1);
	assert check_char_failed("9", p, "ack", 1);
}

#[test]
fn test_list()
{
	let p = "b".lit().list(",".lit());
	
	assert check_str_array_ok("b", p, @~[@~"b"]);
	assert check_str_array_ok("b,b", p, @~[@~"b", @~"b"]);
	assert check_str_array_ok("b,b,b", p, @~[@~"b", @~"b", @~"b"]);
	assert check_str_array_ok("b,b,c", p, @~[@~"b", @~"b"]);
	
	assert check_str_array_failed("", p, "'b'", 1);
	assert check_str_array_failed("c", p, "'b'", 1);
}

#[test]
fn test_parse()
{
	let p = "<".lit().s0().then("foo".lit().s0()).then(">".lit()).err("bracketed foo");
	
	match p.parse(@~"unit test", ~"< foo\t>")
	{
		result::Ok(s) =>
		{
			if s != @~">"
			{
				io::stderr().write_line(fmt!("'>' but found '%s'.", *s));
				assert false;
			}
		}
		result::Err({file, line, col, mesg}) =>
		{
			util::ignore(file);
			io::stderr().write_line(fmt!("Error '%s' on line %u and col %u.", *mesg, line, col));
			assert false;
		}
	}
	
	assert check_str_failed("<foo", p, "'>'", 1);
	match p.parse(@~"unit test", ~"< \n\nfoo\tx")
	{
		result::Ok(s) =>
		{
			io::stderr().write_line(fmt!("Somehow parsed '%s'.", *s));
			assert false;
		}
		result::Err({file, line, col, mesg}) =>
		{
			assert file == @~"unit test";
			assert line == 3u;
			assert col == 5u;
			assert mesg == @~"'>'";
		}
	}
}

#[test]
fn test__r0()
{
	let p = "b".lit().r0();
	
	assert check_str_array_ok("", p, @~[]);
	assert check_str_array_ok("b", p, @~[@~"b"]);
	assert check_str_array_ok("bb", p, @~[@~"b", @~"b"]);
	assert check_str_array_ok("bbb", p, @~[@~"b", @~"b", @~"b"]);
	assert check_str_array_ok("c", p, @~[]);
}

#[test]
fn test__r1()
{
	let p = "b".lit().r1().err("b's");
	
	assert check_str_array_ok("b", p, @~[@~"b"]);
	assert check_str_array_ok("bb", p, @~[@~"b", @~"b"]);
	assert check_str_array_ok("bbb", p, @~[@~"b", @~"b", @~"b"]);
	
	assert check_str_array_failed("", p, "b's", 1);
	assert check_str_array_failed("c", p, "b's", 1);
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
fn test_optional()
{
	let p = seq3_ret_str("a".lit(), "b".lit().optional(), "c".lit());
	
	assert check_str_ok("abc", p, "abc");
	assert check_str_ok("ac", p, "ac");
	assert check_str_failed("ad", p, "'c'", 1);
	assert check_str_failed("dbe", p, "'a'", 1);
}

#[test]
fn test_or_v()
{
	let p = or_v(@~["a".lit(), "bb".lit(), "c".lit()]);
	
	assert check_str_ok("a", p, "a");
	assert check_str_ok("bb", p, "bb");
	assert check_str_ok("c", p, "c");
	assert check_str_ok("ca", p, "c");
	assert check_str_failed("", p, "'a' or 'bb' or 'c'", 1);
	
	let text = chars_with_eot("bz");
	let result = p({file: @~"unit test", text: text, index: 0u, line: 1});
	assert result::get_err(result).old_state.index == 0u;
}

#[test]
fn test_s0()
{
	let p = "x".lit().s0().then("y".lit());
	
	assert check_str_ok("xy", p, "y");
	assert check_str_ok("x y", p, "y");
	assert check_str_ok("x \n\t y", p, "y");
	
	assert check_str_failed("x z", p, "'y'", 1);
	assert check_str_failed("x\nz", p, "'y'", 2);
	assert check_str_failed("x\n\r\nz", p, "'y'", 3);
}

#[test]
fn test_s1()
{
	let p = "x".lit().s1().then("y".lit());
	
	assert check_str_ok("x y", p, "y");
	assert check_str_ok("x \n\t y", p, "y");
	
	assert check_str_failed("xy", p, "whitespace", 1);
	assert check_str_failed("x z", p, "'y'", 1);
	assert check_str_failed("x\nz", p, "'y'", 2);
	assert check_str_failed("x\n\r\nz", p, "'y'", 3);
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
