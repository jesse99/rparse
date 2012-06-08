import test_helpers::*;
import c99_parser::*;

#[test]
fn test_s0()
{
	let p = "x".lit().s0().then("y".lit());
	
	assert check_str_ok("xy", p, "y");
	assert check_str_ok("x y", p, "y");
	assert check_str_ok("x \n\t y", p, "y");
	
	assert check_str_failed("x z", p, "Expected 'y'", 1);
	assert check_str_failed("x\nz", p, "Expected 'y'", 2);
	assert check_str_failed("x\n\r\nz", p, "Expected 'y'", 3);
}

#[test]
fn test_s1()
{
	let p = "x".lit().s1().then("y".lit());
	
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
	let p = match1(is_digit).tag("Expected digits");
	assert check_str_ok("123", p, "123");
	assert check_str_ok("123x", p, "123");
	assert check_str_failed("", p, "Expected digits", 1);
	assert check_str_failed(">", p, "Expected digits", 1);
}

#[test]
fn test_text()
{
	let p = "<".lit();
	assert check_str_ok("<", p, "<");
	assert check_str_failed("", p, "Expected '<'", 1);
	assert check_str_failed(">", p, "Expected '<'", 1);
	
	let p = "++".lit();
	assert check_str_ok("++", p, "++");
	assert check_str_failed("+-", p, "Expected '++'", 1);
	assert check_str_failed("", p, "Expected '++'", 1);
	assert check_str_failed(">", p, "Expected '++'", 1);
}

#[test]
fn test_literalv()
{
	let p = "inf".litv(1000);				// 1000 is pretty big…
	assert check_int_ok("inf", p, 1000);
	assert check_int_failed("", p, "Expected 'inf'", 1);
	assert check_int_failed("in", p, "Expected 'inf'", 1);
	assert check_int_ok("infinite", p, 1000);
}

#[test]
fn test_everything()
{
	let s = return(0).s0();
	let p = everything(decimal_number(), s);
	
	assert check_int_ok("2", p, 2);
	assert check_int_ok("   \t3", p, 3);
	assert check_int_failed("2 ", p, "Expected EOT", 1);
	assert check_int_failed("\t2\n", p, "Expected EOT", 1);
}
