use misc::*;
use parsers::*;
use test_helpers::*;

#[test]
fn test_fails()
{
	let p = fails::<char>("ack");
	
	assert check_char_failed("", p, "ack", 1);
	assert check_char_failed("9", p, "ack", 1);
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
