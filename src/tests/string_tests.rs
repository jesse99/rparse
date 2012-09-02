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
	assert check_str_failed("FoO", p, "'foo'", 1);
	assert check_str_failed("FOO", p, "'foo'", 1);
}

#[test]
fn test_liti()
{
	let p = "foo".liti();
	
	assert check_str_ok("foo", p, "foo");
	assert check_str_ok("foO", p, "foO");
	assert check_str_ok("FOO", p, "FOO");
	assert check_str_ok("foo-shizzle", p, "foo");
	assert check_str_failed("", p, "'foo'", 1);
	assert check_str_failed("bar", p, "'foo'", 1);
	assert check_str_failed("pseudo foo", p, "'foo'", 1);
}

#[test]
fn test_match0()
{
	let p = match0(is_alpha);
	
	assert check_str_ok("foo", p, "foo");
	assert check_str_ok("foo-bar", p, "foo");
	assert check_str_ok("34", p, "");
	assert check_str_ok("", p, "");
}

#[test]
fn test_match1()
{
	let p = match1(is_alpha);
	
	assert check_str_ok("foo", p, "foo");
	assert check_str_ok("foo-bar", p, "foo");
	assert check_str_failed("", p, "", 1);
	assert check_str_failed("34", p, "", 1);
}

#[test]
fn test_match1_0()
{
	let p = match1_0(is_alpha, is_alphanum);
	
	assert check_str_ok("foo", p, "foo");
	assert check_str_ok("foo23z", p, "foo23z");
	assert check_str_failed("", p, "", 1);
	assert check_str_failed("34foo", p, "", 1);
}
