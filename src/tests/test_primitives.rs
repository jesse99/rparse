// Test selected individual parse functions (the other tests suffice for most functions).
import test_helpers::*;

#[test]
fn test_s()
{
	let s = space_zero_or_more(_);

	assert check_ok("", s, 0, 1, 0);
	assert check_ok("x x", s, 0, 1, 0);
	assert check_ok("   ", s, 0, 1, 0);
	assert check_ok("\t\t\n ", s, 0, 2, 0);
	assert check_ok("\t\t\r ", s, 0, 2, 0);
	assert check_ok("\t\t\r\n ", s, 0, 2, 0);
}

#[test]
fn test_spaces()
{
	let s = space_one_or_more(_);

	assert check_err("", s, "expected whitespace", 1, 0);
	assert check_err("x x", s, "expected whitespace", 1, 0);
	assert check_ok("   ", s, 0, 1, 0);
	assert check_ok("\t\t\n ", s, 0, 2, 0);
	assert check_ok("\t\t\r ", s, 0, 2, 0);
	assert check_ok("\t\t\r\n ", s, 0, 2, 0);
}
