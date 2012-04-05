// Tests each individual parse function.
import test_helpers::*;

#[test]
fn test_s()
{
	assert check_ok("", s, (), 1);
	assert check_ok("x x", s, (), 1);
	assert check_ok("   ", s, (), 1);
	assert check_ok("\t\t\n ", s, (), 2);
	assert check_ok("\t\t\r ", s, (), 2);
	assert check_ok("\t\t\r\n ", s, (), 2);
}

#[test]
fn test_spaces()
{
	assert check_err("", spaces, "expected whitespace", 1);
	assert check_err("x x", spaces, "expected whitespace", 1);
	assert check_ok("   ", spaces, (), 1);
	assert check_ok("\t\t\n ", spaces, (), 2);
	assert check_ok("\t\t\r ", spaces, (), 2);
	assert check_ok("\t\t\r\n ", spaces, (), 2);
}
