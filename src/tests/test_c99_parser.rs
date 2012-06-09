import test_helpers::*;

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
fn test_decimal_number()
{
	let p = decimal_number();
	
	assert check_int_ok("1", p, 1);
	assert check_int_ok("123", p, 123);
	assert check_int_ok("123x", p, 123);
	assert check_int_failed("+78", p, "Expected decimal number", 1);
	assert check_int_failed("", p, "Expected decimal number", 1);
	assert check_int_failed("in", p, "Expected decimal number", 1);
}

#[test]
fn test_octal_number()
{
	let p = octal_number();
	
	assert check_int_ok("01", p, 1);
	assert check_int_ok("010", p, 8);
	assert check_int_ok("012", p, 10);
	assert check_int_failed("1", p, "Expected octal number", 1);
	assert check_int_failed("in", p, "Expected octal number", 1);
	assert check_int_failed("0777777777777777777777777", p, "Octal number is too large", 1);
}

#[test]
fn test_hex_number()
{
	let p = hex_number();
	
	assert check_int_ok("0x2", p, 2);
	assert check_int_ok("0xa", p, 10);
	assert check_int_ok("0xF", p, 15);
	assert check_int_ok("0x10", p, 16);
	assert check_int_ok("0xff", p, 255);
	assert check_int_ok("0X80", p, 128);
	assert check_int_failed("1", p, "Expected hex number", 1);
	assert check_int_failed("0xx", p, "Expected hex number", 1);
}

#[test]
fn test_float_number()
{
	let p = float_number();
	
	assert check_float_ok("0.1", p, 0.1);
	assert check_float_ok("0.1e2", p, 10.0);
	assert check_float_ok("0.1e-1", p, 0.01);
	assert check_float_ok("2.", p, 2.0);
	assert check_float_ok("2.e3", p, 2000.0);
	assert check_float_ok("1e3", p, 1000.0);
	assert check_float_failed("x", p, "Expected float number", 1);
	assert check_float_failed("0", p, "Expected float number", 1);
	assert check_float_failed("0x.0", p, "Expected float number", 1);
}

#[test]
fn test_char_literal()
{
	let p = char_literal();
	
	assert check_char_ok("'x'", p, 'x');
	assert check_char_ok("'\\n'", p, '\n');
	assert check_char_ok("'\\52'", p, '*');
	assert check_char_ok("'\\x2A'", p, '*');
	assert check_char_ok("'\\u002A'", p, '*');
	assert check_char_failed("'\\q'", p, "Expected escape character", 1);
	assert check_char_failed("'xx'", p, "Expected '''", 1);
}

#[test]
fn test_string_literal()
{
	let p = string_literal();
	
	assert check_str_ok("\"\"", p, "");
	assert check_str_ok("\"xyz\"", p, "xyz");
	assert check_str_ok("\"a\\nx\"", p, "a\nx");
	assert check_str_failed("\"xx", p, "Expected '\"'", 1);
}

#[test]
fn test_comment()
{
	let p = comment();
	
	assert check_str_ok("/**/", p, "");
	assert check_str_ok("/* blah */", p, " blah ");
	assert check_str_failed("/* xxx\nyyy\nzz", p, "Expected '*/'", 3);
}
