use misc::*;
use parsers::*;
use test_helpers::*;

#[test]
fn test_lit()
{
	let p = "foo".lit();
	
	assert check_str_ok("foo", p, "foo");
	assert check_str_ok("foo-shizzle", p, "foo");
	assert check_str_failed("", p, "'foo'", 1);
	assert check_str_failed("bar", p, "'foo'", 1);
	assert check_str_failed("pseudo foo", p, "'foo'", 1);
}
