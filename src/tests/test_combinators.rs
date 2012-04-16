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
