import misc::*;
import parsers::*;
import primitives::*;
import test_helpers::*;

#[test]
fn test_space0()
{
	let p = text("x").space0()._then(text("y"));
	
	assert check_str_ok("xy", p, "y");
	assert check_str_ok("x y", p, "y");
	assert check_str_ok("x \n\t y", p, "y");
	
	assert check_str_failed("x z", p, "Expected 'y'", 1);
	assert check_str_failed("x\nz", p, "Expected 'y'", 2);
	assert check_str_failed("x\n\r\nz", p, "Expected 'y'", 3);
}

#[test]
fn test_space1()
{
	let p = text("x").space1()._then(text("y"));
	
	assert check_str_ok("x y", p, "y");
	assert check_str_ok("x \n\t y", p, "y");
	
	assert check_str_failed("xy", p, "Expected whitespace", 1);
	assert check_str_failed("x z", p, "Expected 'y'", 1);
	assert check_str_failed("x\nz", p, "Expected 'y'", 2);
	assert check_str_failed("x\n\r\nz", p, "Expected 'y'", 3);
}

#[test]
fn test_match1()
{
	let p = match1(is_digit, "Expected digits");
	assert check_str_ok("123", p, "123");
	assert check_str_ok("123x", p, "123");
	assert check_str_failed("", p, "Expected digits", 1);
	assert check_str_failed(">", p, "Expected digits", 1);
}

#[test]
fn test_text()
{
	let p = text("<");
	assert check_str_ok("<", p, "<");
	assert check_str_failed("", p, "Expected '<'", 1);
	assert check_str_failed(">", p, "Expected '<'", 1);
	
	let p = text("++");
	assert check_str_ok("++", p, "++");
	assert check_str_failed("+-", p, "Expected '++'", 1);
	assert check_str_failed("", p, "Expected '++'", 1);
	assert check_str_failed(">", p, "Expected '++'", 1);
}

#[test]
fn test_literal()
{
	let p = literal("inf", 1000);				// 1000 is pretty big…
	assert check_int_ok("inf", p, 1000);
	assert check_int_failed("", p, "Expected 'inf'", 1);
	assert check_int_failed("in", p, "Expected 'inf'", 1);
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

#[test]
fn test_everything()
{
	let s = return(0).space0();
	let p = everything(integer(), s);
	
	assert check_int_ok("2", p, 2);
	assert check_int_ok("   \t3", p, 3);
	assert check_int_failed("2 ", p, "Expected EOT", 1);
	assert check_int_failed("\t2\n", p, "Expected EOT", 1);
}
