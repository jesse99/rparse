use misc::*;
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

#[test]
fn test_anyc()
{
	let p = "aeiou".anyc();
	
	assert check_char_ok("a", p, 'a');
	assert check_char_ok("e", p, 'e');
	assert check_char_ok("u", p, 'u');
	assert check_char_failed("", p, "[aeiou]", 1);
	assert check_char_failed("9", p, "[aeiou]", 1);
	assert check_char_failed("z", p, "[aeiou]", 1);
}

#[test]
fn test_noc()
{
	let p = "aeiou".noc();
	
	assert check_char_ok("9", p, '9');
	assert check_char_ok("z", p, 'z');
	assert check_char_failed("", p, "[^aeiou]", 1);
	assert check_char_failed("a", p, "[^aeiou]", 1);
	assert check_char_failed("e", p, "[^aeiou]", 1);
	assert check_char_failed("u", p, "[^aeiou]", 1);
}
