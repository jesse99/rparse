import basis::*;
import generators::*;
import misc::*;
import test_helpers::*;

#[test]
fn test_match1()
{
	let p = match1(is_digit, "digits");
	assert check_str_ok("123", p, "123");
	assert check_str_ok("123x", p, "123");
	assert check_str_failed("", p, "digits", 1);
	assert check_str_failed(">", p, "digits", 1);
}

#[test]
fn test_text()
{
	let p = text("<");
	assert check_str_ok("<", p, "<");
	assert check_str_failed("", p, "'<'", 1);
	assert check_str_failed(">", p, "'<'", 1);
	
	let p = text("++");
	assert check_str_ok("++", p, "++");
	assert check_str_failed("+-", p, "'++'", 1);
	assert check_str_failed("", p, "'++'", 1);
	assert check_str_failed(">", p, "'++'", 1);
}

#[test]
fn test_literal()
{
	let p = literal("inf", 1000);				// 1000 is pretty big…
	assert check_int_ok("inf", p, 1000);
	assert check_int_failed("", p, "'inf'", 1);
	assert check_int_failed("in", p, "'inf'", 1);
	assert check_int_ok("infinite", p, 1000);
}

#[test]
fn test_integer()
{
	let p = integer();
	
	assert check_int_ok("1", p, 1);
	assert check_int_ok("123", p, 123);
	assert check_int_ok("123x", p, 123);
	assert check_int_ok("+78", p, 78);
	assert check_int_ok("-14", p, -14);
	assert check_int_failed("", p, "'+' or '-' or digits", 1);
	assert check_int_failed("in", p, "'+' or '-' or digits", 1);
}

#[test]
fn test_identifier()
{
	let p = identifier();
	assert check_str_ok("hey", p, "hey");
	assert check_str_ok("hey_there", p, "hey_there");
	assert check_str_ok("hey there", p, "hey");
	assert check_str_ok("spanky123xy", p, "spanky123xy");
	assert check_str_failed("", p, "identifier", 1);
}
