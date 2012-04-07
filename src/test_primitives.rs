// Test selected individual parse functions (the other tests suffice for most functions).
import test_helpers::*;

#[cfg(test)]
fn s_i(input: state<int>) -> status<int>
{
	ret s(input);
}

#[cfg(test)]
fn spaces_i(input: state<int>) -> status<int>
{
	ret spaces(input);
}

#[test]
fn test_s()
{
	assert check_ok("", s_i, 0, 1, 0);
	assert check_ok("x x", s_i, 0, 1, 0);
	assert check_ok("   ", s_i, 0, 1, 0);
	assert check_ok("\t\t\n ", s_i, 0, 2, 0);
	assert check_ok("\t\t\r ", s_i, 0, 2, 0);
	assert check_ok("\t\t\r\n ", s_i, 0, 2, 0);
}

#[test]
fn test_spaces()
{
	assert check_err("", spaces_i, "expected whitespace", 1, 0);
	assert check_err("x x", spaces_i, "expected whitespace", 1, 0);
	assert check_ok("   ", spaces_i, 0, 1, 0);
	assert check_ok("\t\t\n ", spaces_i, 0, 2, 0);
	assert check_ok("\t\t\r ", spaces_i, 0, 2, 0);
	assert check_ok("\t\t\r\n ", spaces_i, 0, 2, 0);
}
