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
// implement product
// may want a terms parser
// implement expr
// might want to support unary function calls
fn expr_parser2() -> str_parser<int>
{
	let space = s(_);
	let left_paren = literal(_, "(", space);
	let right_paren = literal(_, ")", space);
	let plus_sign = literal(_, "+", space);
	let minus_sign = literal(_, "-", space);
	let int_literal = integer(_, space);
	let expr_ptr = @mut fails(_);
	let expr_ref = cyclic(_, expr_ptr);
	
	// sub_expr := '(' expr ')'
	let sub_expr = sequence(_, [left_paren, expr_ref, right_paren], {|results| results[2]});
	
	// term := [-+]? (integer | sub_expr)
	let term = alternative(_, [
		sequence(_, [plus_sign, sub_expr], {|results| results[2]}),
		sequence(_, [minus_sign, sub_expr], {|results| -results[2]}),
		int_literal,
		sub_expr
	]);
	
	// product := term ([*/] term)*
	// expr := product ([+-] product)*
	// start := s expr
	let expr = term;
	*expr_ptr = expr;
	
	ret everything("unit test", space, expr, 0, _);
}

#[test]
fn test_term()
{
	let expr = expr_parser2();
	
	assert check_err_str("", expr, "expected '+' or expected '-' or expected an integer or expected '('", 1);
	assert expr_ok("23", expr, 23, 1);
	assert expr_ok(" 57   ", expr, 57, 1);
	assert expr_ok("\t\t\n-100", expr, -100, 2);
	assert expr_ok("+1", expr, 1, 1);
	assert check_err_str("+", expr, "expected '('", 1);
	assert check_err_str(" 57   200", expr, "expected EOT but found '200'", 1);

	assert expr_ok("(23)", expr, 23, 1);
	assert expr_ok("((23))", expr, 23, 1);
	assert check_err_str("((23)", expr, "expected ')'", 1);
	
	assert expr_ok("-(23)", expr, -23, 1);
	assert expr_ok("+(5)", expr, 5, 1);
}
