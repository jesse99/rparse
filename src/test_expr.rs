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

fn expr_parser() -> str_parser<int>
{
	let s = space_zero_or_more(_);
	let left_paren = literal(_, "(", s);
	let right_paren = literal(_, ")", s);
	let plus_sign = literal(_, "+", s);
	let minus_sign = literal(_, "-", s);
	let mult_sign = literal(_, "*", s);
	let div_sign = literal(_, "/", s);
	let int_literal = integer(_, s);
	let expr_ptr = @mut fails(_);
	let expr_ref = forward_ref(_, expr_ptr);	
	
	// sub_expr := '(' expr ')'
	let sub_expr = sequence(_, [left_paren, expr_ref, right_paren], {|results| results[2]});
	
	// factor := [-+]? (integer | sub_expr)
	let factor = alternative(_, [
		sequence(_, [plus_sign, sub_expr], {|results| results[2]}),
		sequence(_, [minus_sign, sub_expr], {|results| -results[2]}),
		int_literal,
		sub_expr
	]);
	
	// term := factor ([*/] factor)*
	let mult = fn@ (&&x: int, &&y: int) -> int {ret x * y};	// generic arguments are currently always passed by pointer so we need the lame && sigil
	let div = fn@ (&&x: int, &&y: int) -> int {ret x / y};
	let term = binary_op(_, factor, [
		(mult_sign, factor, mult),
		(div_sign, factor, div)
	]);
	
	// expr := term ([+-] term)*
	let add = fn@ (&&x: int, &&y: int) -> int {ret x + y};
	let sub = fn@ (&&x: int, &&y: int) -> int {ret x - y};
	let expr = binary_op(_, term, [
		(plus_sign, term, add),
		(minus_sign, term, sub)
	]);
	*expr_ptr = expr;
	
	// start := s expr
	let start = everything("unit test", expr, s, 0, _);
	ret start;
}

#[test]
fn test_factor()
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

#[test]
fn test_term()
{
	let expr = expr_parser();
	
	assert expr_ok("2*3", expr, 6, 1);
	assert expr_ok(" 4 / 2   ", expr, 2, 1);
	assert check_err_str("4 * ", expr, "expected EOT but found '* '", 1);
	assert check_err_str("4 ** 1", expr, "expected EOT but found '** 1'", 1);
	assert check_err_str("4 % 1", expr, "expected EOT but found '% 1'", 1);
	
	assert expr_ok("2 * 3 / 6", expr, 1, 1);
}

#[test]
fn test_expr()
{
	let expr = expr_parser();
	
	assert expr_ok("3+2", expr, 5, 1);
	assert expr_ok(" 3\t-2  ", expr, 1, 1);
	assert expr_ok("2 + 3*4", expr, 14, 1);
	assert expr_ok("(2 + 3)*4", expr, 20, 1);
}
