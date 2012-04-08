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

// TODO:
// might want to try adding a curry operator (or function)
// implement product
// may want a terms parser
#[cfg(test)]
fn expr_parser() -> str_parser<int>
{
	// pexpr := '(' expr ')'
	// term := [-+] (integer | pexpr)
	// product := term ([*/] term)*
	// expr := product ([+-] product)*
	// start := s expr
	let pe = @mut fails(_);
	let pexpr = sequence(_, [
		literal(_, "(", s(_)), 
		cyclic(_, pe),
		literal(_, ")", s(_))]);
	
	let term = alternative(_, [integer(_, s(_)), pexpr]);
	let expr = term;
	*pe = expr;
	
	ret everything("unit test", _, s(_), expr, 0);
}

#[test]
fn test_term()
{
	let expr = expr_parser();
	
	assert check_err_str("", expr, "expected an integer or expected '('", 1);
	assert expr_ok("23", expr, 23, 1);
	assert expr_ok(" 57   ", expr, 57, 1);
	assert expr_ok("\t\t\n-100", expr, -100, 2);
	assert expr_ok("+1", expr, 1, 1);
	assert check_err_str("+", expr, "expected an integer", 1);
	assert check_err_str(" 57   200", expr, "expected EOT but found '200'", 1);

	assert expr_ok("(23)", expr, 23, 1);
	assert expr_ok("((23))", expr, 23, 1);
	assert check_err_str("((23)", expr, "expected ')'", 1);
	
	// TODO: test leading sign
}
