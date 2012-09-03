## Description
rparse is a parser combinator library written in the [Rust](http://www.rust-lang.org) programming
language. The library consists of parse functions that can be composed together to create arbitrarily 
powerful parsers. The design is such that it is easy for users to define their own parse functions (e.g. 
to define a custom whitespace parser which includes support for a particular comment style).

The parse functions all take a state record as input containing the text to be parsed as well as how much 
has been parsed. They return a result that is either passed or failed. If passed the result includes a new 
state record and a generic T value. If failed the result consists of the input state and an error string.

The library has been tested with Rust from github as of Sep 1 2012 (i.e. pre 0.4).

## Example
Here is an example of a simple parser which can be used to evaluate mathematical expressions.

    import rparse::*;
    
    fn expr_parser() -> Parser<int>
    {
        let int_literal = decimal_number().err("number").s0();
        
        // Parenthesized expressions require a forward reference to the expr parser
        // so we initialize a function pointer to something of the right type, create
        // a parser using the parser expr_ptr points to, and fixup expr_ptr later.
        let expr_ptr = @mut ret(0i);
        let expr_ref = forward_ref(expr_ptr);
        
        // sub_expr := [-+]? '(' expr ')'
        // The err function provides better error messages if the factor parser fails
        // on the very first character.
        let sub_expr = or_v(@~[
            seq4_ret2("+".s0(), "(".s0(), expr_ref, ")".s0()),
            seq4_ret2("-".s0(),  "(".s0(), expr_ref, ")".s0()).thene(|v| ret(-v) ),
            seq3_ret1(               "(".s0(), expr_ref, ")".s0())]).err("sub-expression");
        
        // factor := integer | sub_expr
        let factor = int_literal.or(sub_expr);
        
        // term := factor ([*/] factor)*
        let term = do factor.chainl1("*".s0().or("/".s0()))
            |lhs, op, rhs| {if op == @~"*" {lhs*rhs} else {lhs/rhs}};
        
        // expr := term ([+-] term)*
        let expr = term.chainl1("+".s0().or("-".s0()),
            |lhs, op, rhs| {if op == @~"+" {lhs + rhs} else {lhs - rhs}}).err("expression");
        *expr_ptr = expr;
        
        // start := s0 expr EOT
        let s = ret(0).s0();
        expr.everything(s)
    }

Usage looks like this:

    fn test_usage()
    {
        match expr_parser().parse(@~"test", ~"2+3*5")
        {
            result::Ok(value) => assert value == 17,
            result::Err(_) => assert false,
        }
    }
