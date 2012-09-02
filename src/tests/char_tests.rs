// Test selected individual parse functions (the other tests suffice for most functions).
//use io;
//use io::WriterUtil;
//use c99_parsers::*;
//use combinators::*;
//use generic_parsers::*;
use misc::*;
//use result::*;
use parsers::*;
use test_helpers::*;

#[test]
fn test_anycp()
{
	let p = anycp(is_alpha);
	
	assert check_char_ok("a", p, 'a');
	assert check_char_ok("Z", p, 'Z');
	assert check_char_failed("", p, "", 1);
	assert check_char_failed("9", p, "", 1);
}
