// Test a grammar capable of evaluating simple mathematical expressions.
import test_helpers::*;

fn expr_parser() -> parser<int>
{
	let int_literal = decimal_number().s0();
	
	// Parenthesized expressions require a forward reference to the expr parser
	// so we initialize a function pointer to something of the right type, create
	// a parser using the parser expr_ptr points to, and fixup expr_ptr later.
	let expr_ptr = @mut return(0);
	let expr_ref = forward_ref(expr_ptr);
	
	// sub_expr := [-+]? '(' expr ')'
	// The err function provides better error messages if the factor parser fails
	// on the very first character.
	let sub_expr = or_v([
		seq4_ret2("+".s0(), "(".s0(), expr_ref, ")".s0()),
		seq4_ret2("-".s0(),  "(".s0(), expr_ref, ")".s0()).thene({|v| return(-v)}),
		seq3_ret1(             "(".s0(), expr_ref, ")".s0())]/~).err("sub-expression");
	
	// factor := integer | sub_expr
	let factor = int_literal.or(sub_expr);
	
	// term := factor ([*/] factor)*
	let term = factor.chainl1("*".s0().or("/".s0()))
		{|lhs, op, rhs| if op == "*" {lhs*rhs} else {lhs/rhs}};
	
	// expr := term ([+-] term)*
	let expr = term.chainl1("+".s0().or("-".s0()))
		{|lhs, op, rhs| if op == "+" {lhs + rhs} else {lhs - rhs}};
	*expr_ptr = expr;
	
	// start := s0 expr EOT
	let s = return(0).s0();
	expr.everything(s)
}

#[test]
fn test_factor()
{
	let p = expr_parser();
	
	assert check_int_failed("", p, "decimal number or sub-expression", 1);
	assert check_int_ok("23", p, 23);
	assert check_int_ok(" 57   ", p, 57);
	assert check_int_failed("+", p, "'('", 1);
	assert check_int_failed(" 57   200", p, "EOT", 1);
	
	// TODO: https://github.com/mozilla/rust/issues/2546
	//assert check_int_failed("9999999999999999999999", p, "'9999999999999999999999' is out of range", 1);
	
	assert check_int_ok("(23)", p, 23);
	assert check_int_ok("((23))", p, 23);
	assert check_int_failed("(23", p, "')'", 1);
	assert check_int_failed("((23)", p, "')'", 1);
	
	assert check_int_ok("-(23)", p, -23);
	assert check_int_ok("+(5)", p, 5);
}

#[test]
fn test_term()
{
	let p = expr_parser();
	
	assert check_int_ok("2*3", p, 6);
	assert check_int_ok(" 4 / 2   ", p, 2);
	assert check_int_failed("4 * ", p, "EOT", 1);
	assert check_int_failed("4 ** 1", p, "EOT", 1);
	assert check_int_failed("4 % 1", p, "EOT", 1);
	
	assert check_int_ok("2 * 3 / 6", p, 1);
}

#[test]
fn test_expr()
{
	let p = expr_parser();
	
	assert check_int_ok("3+2", p, 5);
	assert check_int_ok(" 3\t-2  ", p, 1);
	assert check_int_ok("2 + 3*4", p, 14);
	assert check_int_ok("(2 + 3)*4", p, 20);
}

#[test]
fn test_usage()
{
	alt expr_parser().parse("test", "2+3*5")
	{
		result::ok(value)
		{
			assert value == 17;
		}
		result::err({file, line, col, mesg})
		{
			assert false;
		}
	}
}

// This doesn't really test anything that we don't test elsewhere, but it's convenient
// to use when examining how well rparse handles logging.
#[test]
fn test_log()
{
	let p = expr_parser();
	
	assert check_int_failed("2+3**5", p, "EOT", 1);
}
