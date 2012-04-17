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

## Example
Here is an example of a simple parser which can be used to evaluate mathematical expressions.

    fn expr_parser() -> parser<int>
    {
        // Create parsers for punctuation and integer literals. All of these
        // parsers allow for zero or more trailing whitespace characters.
        let plus_sign = text("+").space0();
        let minus_sign = text("-").space0();
        let mult_sign = text("*").space0();
        let div_sign = text("/").space0();
        let left_paren = text("(").space0();
        let right_paren = text(")").space0();
        let int_literal = integer().space0();
        
        // Parenthesized expressions require a forward reference to the expr parser
        // so we initialize a function pointer to something of the right type, create
        // a parser using the parser expr_ptr points to, and fixup expr_ptr later.
        let expr_ptr = @mut return(0);
        let expr_ref = forward_ref(expr_ptr);
        
        // sub_expr := [-+]? '(' expr ')'
        let sub_expr = alternative([
            sequence4(plus_sign, left_paren, expr_ref, right_paren) {|_a, _b, c, _d| c},
            sequence4(minus_sign, left_paren, expr_ref, right_paren) {|_a, _b, c, _d| -c},
            sequence3(left_paren, expr_ref, right_paren) {|_a, b, _c| b}]);
        
        // factor := integer | sub_expr
        // The tag provides better error messages if the factor parser fails
        // on the very first character.
        let factor = int_literal.or(sub_expr).tag("integer or sub-expression");
        
        // term := factor ([*/] factor)*
        let term = factor.chainl1(mult_sign.or(div_sign))
            {|lhs, op, rhs| if op == "*" {lhs*rhs} else {lhs/rhs}};
        
        // expr := term ([+-] term)*
        let expr = term.chainl1(plus_sign.or(minus_sign))
            {|lhs, op, rhs| if op == "+" {lhs + rhs} else {lhs - rhs}};
        *expr_ptr = expr;
        
        // start := s expr
        // everything is a parser that accepts leading whitespace, followed by
        // an abitrary parser, followed by EOT. (The s syntax is a little goofy
        // because the space0 can't rely on expr to figure out which type to use).
        let s = return(0).space0();
        let start = expr.everything(s);
        
        ret start;
    }

Usage looks like this:

    fn eval(file: str, text: str) -> option::option<int>
    {
        let parser = expr_parser();
        alt parser.parse(file, text)    // file is returned in err_state to make error reporting easier
        {
            result::ok(pass)
            {
                ret option::some(pass.value);
            }
            result::err(failure)
            {
                io::stderr().write_line(#fmt["Error '%s' on line %u", failure.mesg, failure.err_state.line]);
                ret option::none;
            }
        }
    }
