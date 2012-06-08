import c99_parser::*;
import test_helpers::*;

#[test]
fn test_integer()
{
	let p = integer();
	
	assert check_int_ok("1", p, 1);
	assert check_int_ok("123", p, 123);
	assert check_int_ok("123x", p, 123);
	assert check_int_ok("+78", p, 78);
	assert check_int_ok("-14", p, -14);
	assert check_int_failed("", p, "Expected '+' or '-' or digits", 1);
	assert check_int_failed("in", p, "Expected '+' or '-' or digits", 1);
}

#[test]
fn test_identifier()
{
	let p = identifier();
	assert check_str_ok("hey", p, "hey");
	assert check_str_ok("hey_there", p, "hey_there");
	assert check_str_ok("hey there", p, "hey");
	assert check_str_ok("spanky123xy", p, "spanky123xy");
	assert check_str_failed("", p, "Expected identifier", 1);
}

