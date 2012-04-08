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
// do a commit
// may want to get rid of macros (try stuff like `let left_paren = literal(…)`
// implement product
// may want a terms parser
// implement expr
// might want to support unary function calls
#[cfg(test)]
fn expr_parser() -> str_parser<int>
{
	// product := term ([*/] term)*
	// expr := product ([+-] product)*
	// start := s expr
	let space = s(_);
	let expr_ptr = @mut fails(_);
	
	// sub_expr := '(' expr ')'
	let sub_expr = sequence(_, [#literal["("], #cyclic[expr_ptr], #literal[")"]], {|results| results[2]});
	
	// term := [-+]? (integer | sub_expr)
	let term = #alternative[
		sequence(_, [#literal["+"], sub_expr], {|results| results[2]}),
		sequence(_, [#literal["-"], sub_expr], {|results| -results[2]}),
		#integer[],
		sub_expr
	];
	let expr = term;
	*expr_ptr = expr;
	
	ret #everything["unit test", expr];
}

#[test]
fn test_term()
{
	let expr = expr_parser();
	
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
