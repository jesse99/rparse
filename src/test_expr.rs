// Test a grammar capable of evaluating simple mathematical expressions.
import io;
import io::writer_util;
import test_helpers::*;

#[cfg(test)]
fn expr_ok(text: str, parser: str_parser<int>, expected: int, line: int) -> bool
{
	alt parser(text)
	{
		result::ok(answer)
		{
			if answer.value != expected
			{
				io::stderr().write_line(#fmt["Expected %d but found %d", expected, answer.value]);
				ret false;
			}
			if answer.line != line
			{
				io::stderr().write_line(#fmt["Expected line %d but found line %d", line, answer.line]);
				ret false;
			}
			ret true;
		}
		result::err(error)
		{
			io::stderr().write_line(#fmt["Error: %s", error.mesg]);
			ret false;
		}
	}
}

// TODO: is there no way to use s<int>?
#[cfg(test)]
fn s_int(input: state<int>) -> status<int>
{
	ret s(input);
}

#[cfg(test)]
fn expr_parser() -> str_parser<int>
{
	// expr := product ([+-] product)*
	// product := term ([*/] term)*
	// pexpr := '(' expr ')'
	// term := '-'? (integer | pexpr)
	let term = bind integer(_, s_int);
	ret bind everything("unit test", _, s_int, term, 0);
}

#[test]
fn test_term()
{
	let expr = expr_parser();
	
	assert check_err_str("", expr, "expected an integer", 1);
	assert expr_ok("23", expr, 23, 1);
	assert expr_ok(" 57   ", expr, 57, 1);
	assert expr_ok("\t\t\n-100", expr, -100, 2);
	assert expr_ok("+1", expr, 1, 1);
	assert check_err_str("+", expr, "expected an integer", 1);
	assert check_err_str(" 57   200", expr, "expected EOT but found '200'", 1);
}
