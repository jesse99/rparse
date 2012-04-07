// Test selected individual parse functions (the other tests suffice for most functions).
import test_helpers::*;

#[test]
fn test_s()
{
	assert check_ok("", s(_), 0, 1, 0);
	assert check_ok("x x", s(_), 0, 1, 0);
	assert check_ok("   ", s(_), 0, 1, 0);
	assert check_ok("\t\t\n ", s(_), 0, 2, 0);
	assert check_ok("\t\t\r ", s(_), 0, 2, 0);
	assert check_ok("\t\t\r\n ", s(_), 0, 2, 0);
}

#[test]
fn test_spaces()
{
	assert check_err("", spaces(_), "expected whitespace", 1, 0);
	assert check_err("x x", spaces(_), "expected whitespace", 1, 0);
	assert check_ok("   ", spaces(_), 0, 1, 0);
	assert check_ok("\t\t\n ", spaces(_), 0, 2, 0);
	assert check_ok("\t\t\r ", spaces(_), 0, 2, 0);
	assert check_ok("\t\t\r\n ", spaces(_), 0, 2, 0);
}
