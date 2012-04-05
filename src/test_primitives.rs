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

#[test]
fn test_literal()
{
	assert check_err("", bind literal(_, "+", s), "expected '+'", 1);
	assert check_ok("+ x", bind literal(_, "+", s), "+", 1);
	assert check_ok("<<", bind literal(_, "<<", s), "<<", 1);
}

#[test]
fn test_identifier()
{
	assert check_err("", bind identifier(_, s), "expected identifier", 1);
	assert check_ok("begin end", bind identifier(_, s), "begin", 1);
	assert check_ok("beginning", bind identifier(_, s), "beginning", 1);
	assert check_ok("z99_", bind identifier(_, s), "z99_", 1);
	assert check_ok("_", bind identifier(_, s), "_", 1);
}
