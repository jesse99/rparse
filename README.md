## Description
rparse is a parser combinator library written in the [Rust](http://www.rust-lang.org) programming
language. The library consists of parse functions that can be composed together to create arbitrarily 
powerful parsers. The design is such that it is easy for users to define their own parse functions (e.g. 
to define a custom whitespace parser which includes support for a particular comment style).

The parse functions all take a state record as input containing the text to be parsed as well as how much 
has been parsed. They return a result that is either passed or failed. If passed the result includes a new 
state record and a generic T value. If failed the result consists of the input state and an error string.

Most of the built in parse functions take more than just a state argument. Those extra arguments must 
be bound before the parse function can be composed with other functions. For example, 
`bind(repeat_one, _, hex_digit)` creates a parser that consumes one or more hexadecimal digits.

The library has been tested with Rust from github as of April 2010 (i.e. post 0.2).

## Example
Here is an example of a simple parser which can be used to evaluate mathematical expressions.

    import rparse::*;
    import rparse::types::*;
    
    fn expr_parser() -> parser<int>
    {
        // Create parsers for punctuation and integer literals. All of these
        // parsers allow for zero or more trailing whitespace characters.
        let plus_sign = literal("+").space0();
        let minus_sign = literal("-").space0();
        let mult_sign = literal("*").space0();
        let div_sign = literal("/").space0();
        let left_paren = literal("(").space0();
        let right_paren = literal(")").space0();
        let int_literal = integer().space0();
        
        // Parenthesized expressions require a forward reference to the expr parser
        // so we initialize a function pointer to something of the right type, create
        // a parser using the parser expr_ptr points to, and fixup expr_ptr later.
        let expr_ptr = @mut return(0);
        let expr_ref = forward_ref(expr_ptr); 
        
        // sub_expr := [-+]? '(' expr ')'
        let sub_expr = alternative([
            sequence4(plus_sign, left_paren, expr_ref, right_paren) {|_a, _b, c, _d| result::ok(c)},
            sequence4(minus_sign, left_paren, expr_ref, right_paren) {|_a, _b, c, _d| result::ok(-c)},
            sequence3(left_paren, expr_ref, right_paren) {|_a, b, _c| result::ok(b)}]);
        
        // factor := integer | sub_expr
        // The tag provides better error messages if the factor parser fails
        // on the very first character.
        let factor = int_literal.or(sub_expr).tag("Expected integer or sub-expression");
        
        // term := factor ([*/] factor)*
        let term = factor.chainl1(mult_sign.or(div_sign))
            {|lhs, op, rhs| if op == "*" {lhs*rhs} else {lhs/rhs}};
        
        // expr := term ([+-] term)*
        let expr = term.chainl1(plus_sign.or(minus_sign))
            {|lhs, op, rhs| if op == "+" {lhs + rhs} else {lhs - rhs}};
        *expr_ptr = expr;
        
        // start := space0 expr EOT
        // The s syntax is a little goofy because the space0 comes before 
        // instead of after expr so it needs to be told which type to use.
        let s = return(0).space0();
        everything(expr, s)
    }

Usage looks like this:

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
