import basis::*;
import combinators::*;
import generators::*;
import misc::*;
import test_helpers::*;

#[test]
fn test__then()
{
	let p = text("<")._then(text("foo"))._then(text(">"));
	
	assert check_str_ok("<foo>", p, ">");
	assert check_str_failed("", p, "'<'", 1);
	assert check_str_failed("<", p, "'foo'", 1);
	assert check_str_failed("<foo", p, "'>'", 1);
	assert check_str_failed("<foo-", p, "'>'", 1);
	
	let text = chars_with_eot("<foo-");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;	// if any of the then clauses fails we need to start over
}

#[test]
fn test__repeat0()
{
	let p = text("b").repeat0();
	
	assert check_str_array_ok("", p, []);
	assert check_str_array_ok("b", p, ["b"]);
	assert check_str_array_ok("bb", p, ["b", "b"]);
	assert check_str_array_ok("bbb", p, ["b", "b", "b"]);
	assert check_str_array_ok("c", p, []);
}

#[test]
fn test__repeat1()
{
	let p = text("b").repeat1("b's");
	
	assert check_str_array_ok("b", p, ["b"]);
	assert check_str_array_ok("bb", p, ["b", "b"]);
	assert check_str_array_ok("bbb", p, ["b", "b", "b"]);
	
	assert check_str_array_failed("", p, "b's", 1);
	assert check_str_array_failed("c", p, "b's", 1);
}

#[test]
fn test_list()
{
	let p = text("b").list(text(","));
	
	assert check_str_array_ok("b", p, ["b"]);
	assert check_str_array_ok("b,b", p, ["b", "b"]);
	assert check_str_array_ok("b,b,b", p, ["b", "b", "b"]);
	assert check_str_array_ok("b,b,c", p, ["b", "b"]);
	
	assert check_str_array_failed("", p, "'b'", 1);
	assert check_str_array_failed("c", p, "'b'", 1);
}

#[test]
fn test_space0()
{
	let p = text("x").space0()._then(text("y"));
	
	assert check_str_ok("xy", p, "y");
	assert check_str_ok("x y", p, "y");
	assert check_str_ok("x \n\t y", p, "y");
	
	assert check_str_failed("x z", p, "'y'", 1);
	assert check_str_failed("x\nz", p, "'y'", 2);
	assert check_str_failed("x\n\r\nz", p, "'y'", 3);
}

#[test]
fn test_space1()
{
	let p = text("x").space1()._then(text("y"));
	
	assert check_str_ok("x y", p, "y");
	assert check_str_ok("x \n\t y", p, "y");
	
	assert check_str_failed("xy", p, "whitespace", 1);
	assert check_str_failed("x z", p, "'y'", 1);
	assert check_str_failed("x\nz", p, "'y'", 2);
	assert check_str_failed("x\n\r\nz", p, "'y'", 3);
}

#[test]
fn test_optional()
{
	let p = text("x").optional("z");
	
	assert check_str_ok("x", p, "x");
	assert check_str_ok("b", p, "z");
	assert check_str_ok("", p, "z");
}

#[test]
fn test_alternative()
{
	let p = alternative([text("a"), text("bb"), text("c")]);
	
	assert check_str_ok("a", p, "a");
	assert check_str_ok("bb", p, "bb");
	assert check_str_ok("c", p, "c");
	assert check_str_ok("ca", p, "c");
	assert check_str_failed("", p, "'a' or 'bb' or 'c'", 1);
	
	let text = chars_with_eot("bz");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;
}

pure fn is_identifier_trailer(ch: char) -> bool
{
	ret ch == '?' || ch == '!';
}

#[test]
fn test_sequence()
{
	let prefix = match1(is_identifier_prefix, "identifier");
	let suffix = match1(is_identifier_suffix, "identifier").repeat0();
	let trailer = match1(is_identifier_trailer, "identifier").optional("");
	let p = sequence3(prefix, suffix, trailer, {|a, b, c| a + str::connect(b, "") + c});
	
	assert check_str_ok("hey", p, "hey");
	assert check_str_ok("hey?", p, "hey?");
	assert check_str_ok("hey!", p, "hey!");
	assert check_str_ok("hey_there", p, "hey_there");
	assert check_str_ok("hey there", p, "hey");
	assert check_str_ok("spanky123xy", p, "spanky123xy");
	assert check_str_failed("", p, "identifier", 1);
	
	let p = sequence2(text("a"), text("b"), {|x, y| x+y});
	let text = chars_with_eot("az");
	let result = p({file: "unit test", text: text, index: 0u, line: 1});
	assert get_err(result).old_state.index == 0u;
}

#[test]
fn test_chainl1()
{
	let factor = integer();
	let op = text("*").or(text("/"));
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
	let op = text("*").or(text("/"));
	let p = factor.chainr1(op, {|lhs, op, rhs| if op == "*" {lhs * rhs} else {lhs / rhs}});
	
	assert check_int_ok("2", p, 2);
	assert check_int_ok("2*3", p, 6);
	assert check_int_ok("2*3/4", p, 0);
	assert check_int_ok("2*3/4/2", p, 2);
	assert check_int_ok("2*3-4", p, 6);
}
