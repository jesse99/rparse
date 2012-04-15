//import io;
//import io::writer_util;
//import result = result::result;
import basis::*;
import generators::*;
import misc::*;
import test_helpers::*;

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
	let p = literal("inf", 1000);				// 1000 is pretty big...
	assert check_int_ok("inf", p, 1000);
	assert check_int_failed("", p, "'inf'", 1);
	assert check_int_failed("in", p, "'inf'", 1);
	assert check_int_ok("infinite", p, 1000);
}
