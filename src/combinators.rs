//! Functions and methods used to compose parsers.
//!
//! Note that these functions and methods don't actually consume input (although 
//! the parsers they are invoked with normally will).
use generic_parsers::*;
use parser::*;
use types::*;

// chain_suffix := (op e)*
#[doc(hidden)]
fn chain_suffix<T: copy owned, U: copy owned>(parser: parser<T>, op: parser<U>) -> parser<~[(U, T)]>
{
	let q = op.thene(
	|operator|
	{
		parser.thene(
		|value|
		{
			return((operator, value))
		})
	});
	q.r0()
}

/// chainl1 := e (op e)*
/// 
/// Left associative binary operator. eval is called for each parsed op.
fn chainl1<T: copy owned, U: copy owned>(parser: parser<T>, op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
{
	|input: state| {
		do result::chain(parser(input))
		|pass| {
			match parser.chain_suffix(op)(pass.new_state)
			{
				result::Ok(pass2) =>
				{
					let value = vec::foldl(pass.value, pass2.value, {|lhs, rhs: (U, T)| eval(lhs, rhs.first(), rhs.second())});
					result::Ok({new_state: pass2.new_state, value: value})
				}
				result::Err(failure) =>
				{
					result::Err({old_state: input ,.. failure})
				}
			}
		}
	}
}

/// chainr1 := e (op e)*
/// 
/// Right associative binary operator. eval is called for each parsed op.
fn chainr1<T: copy owned, U: copy owned>(parser: parser<T>, op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
{
	|input: state| {
		do result::chain(parser(input))
		|pass| {
			match parser.chain_suffix(op)(pass.new_state)
			{
				result::Ok(pass2) =>
				{
					if vec::is_not_empty(pass2.value)
					{
						// e1 and [(op1 e2), (op2 e3)]
						let e1 = pass.value;
						let terms = pass2.value;
						
						// e1 and [op1, op2] and [e2, e3]
						let (ops, parsers) = vec::unzip(terms);
						
						// [op1, op2] and [e1, e2] and e3
						let e3 = vec::last(parsers);
						let parsers = ~[e1] + vec::slice(parsers, 0u, vec::len(parsers) - 1u);
						
						// [(e1 op1), (e2 op2)] and e3
						let terms = vec::zip(parsers, ops);
						
						let value = vec::foldr(terms, e3, {|lhs: (T, U), rhs| eval(lhs.first(), lhs.second(), rhs)});
						result::Ok({new_state: pass2.new_state, value: value})
					}
					else
					{
						result::Ok({new_state: pass2.new_state, value: pass.value})
					}
				}
				result::Err(failure) =>
				{
					result::Err({old_state: input ,.. failure})
				}
			}
		}
	}
}

/// Parses with the aid of a pointer to a parser (useful for things like parenthesized expressions).
fn forward_ref<T: copy owned>(parser: @mut parser<T>) -> parser<T>
{
	|input: state| {
		(*parser)(input)
	}
}

/// list := e (sep e)*
/// 
/// Values for each parsed e are returned.
fn list<T: copy owned, U: copy owned>(parser: parser<T>, sep: parser<U>) -> parser<~[T]>
{
	let term = sep.then(parser).r0();
	
	|input: state| {
		do result::chain(parser(input))
		|pass| {
			match term(pass.new_state)
			{
				result::Ok(pass2) =>
				{
					result::Ok({value: ~[pass.value] + pass2.value ,.. pass2})
				}
				result::Err(failure) =>
				{
					result::Err({old_state: input ,.. failure})
				}
			}
		}
	}
}

/// optional := e?
fn optional<T: copy owned>(parser: parser<T>) -> parser<Option<T>>
{
	|input: state| {
		match parser(input)
		{
			result::Ok(pass) =>
			{
				result::Ok({new_state: pass.new_state, value: option::Some(pass.value)})
			}
			result::Err(_failure) =>
			{
				result::Ok({new_state: input, value: option::None})
			}
		}
	}
}

// When using tag it can be useful to use empty messages for interior parsers
// so we need to handle that case.
fn or_mesg(mesg1: ~str, mesg2: ~str) -> ~str
{
	if str::is_not_empty(mesg1) && str::is_not_empty(mesg2)
	{
		mesg1 + " or " + mesg2
	}
	else if str::is_not_empty(mesg1)
	{
		mesg1
	}
	else if str::is_not_empty(mesg2)
	{
		mesg2
	}
	else
	{
		~""
	}
}

/// Returns a parser which first tries parser1, and if that fails, parser2.
fn or<T: copy owned>(parser1: parser<T>, parser2: parser<T>) -> parser<T>
{
	|input: state| {
		do result::chain_err(parser1(input))
		|failure1| {
			do result::chain_err(parser2(input))
			|failure2| {
				if failure1.err_state.index > failure2.err_state.index
				{
					result::Err(failure1)
				}
				else if failure1.err_state.index < failure2.err_state.index
				{
					result::Err(failure2)
				}
				else
				{
					result::Err({mesg: or_mesg(failure1.mesg, failure2.mesg) ,.. failure2})
				}
			}
		}
	}
}

/// or_v := e0 | e1 | …
/// 
/// This is a version of or that is nicer to use when there are more than two alternatives.
fn or_v<T: copy owned>(parsers: ~[parser<T>]) -> parser<T>
{
	// A recursive algorithm would be a lot simpler, but it's not clear how that could
	// produce good error messages.
	assert vec::is_not_empty(parsers);
	
	|input: state| {
		let mut result: Option<status<T>> = None;
		let mut errors = ~[];
		let mut max_index = uint::max_value;
		let mut i = 0u;
		while i < vec::len(parsers) && option::is_none(result)
		{
			match parsers[i](input)
			{
				result::Ok(pass) =>
				{
					result = option::Some(result::Ok(pass));
				}
				result::Err(failure) =>
				{
					if failure.err_state.index > max_index || max_index == uint::max_value
					{
						errors = ~[failure.mesg];
						max_index = failure.err_state.index;
					}
					else if failure.err_state.index == max_index
					{
						vec::push(errors, failure.mesg);
					}
				}
			}
			i += 1u;
		}
		
		if option::is_some(result)
		{
			option::get(result)
		}
		else
		{
			let errs = vec::filter(errors, |s| str::is_not_empty(s));
			let mesg = str::connect(errs, ~" or ");
			result::Err({old_state: input, err_state: {index: max_index ,.. input}, mesg: mesg})
		}
	}
}

/// Succeeds if parser matches input n to m times (inclusive).
fn r<T: copy owned>(parser: parser<T>, n: uint, m: uint) -> parser<~[T]>
{
	|input: state| {
		let mut output = input;
		let mut values = ~[];
		loop
		{
			match parser(output)
			{
				result::Ok(pass) =>
				{
					assert pass.new_state.index > output.index;	// must make progress to ensure loop termination
					output = pass.new_state;
					vec::push(values, pass.value);
				}
				result::Err(failure) =>
				{
					break;
				}
			}
		}
		
		let count = vec::len(values);
		if n <= count && count <= m
		{
			result::Ok({new_state: output, value: values})
		}
		else
		{
			result::Err({old_state: input, err_state: output, mesg: ~""})
		}
	}
}

/// r0 := e*
/// 
/// Values for each parsed e are returned.
fn r0<T: copy owned>(parser: parser<T>) -> parser<~[T]>
{
	r(parser, 0u, uint::max_value)
}

/// r1 := e+
/// 
/// Values for each parsed e are returned.
fn r1<T: copy owned>(parser: parser<T>) -> parser<~[T]>
{
	r(parser, 1u, uint::max_value)
}
/// seq2 := e0 e1
fn seq2_ret0<T0: copy owned, T1: copy owned>(p0: parser<T0>, p1: parser<T1>) -> parser<T0>
{
	seq2(p0, p1, |a0, _a1| result::Ok(a0))
}

/// seq2 := e0 e1
fn seq2_ret1<T0: copy owned, T1: copy owned>(p0: parser<T0>, p1: parser<T1>) -> parser<T1>
{
	seq2(p0, p1, |_a0, a1| result::Ok(a1))
}

/// seq3 := e0 e1 e2
fn seq3_ret0<T0: copy owned, T1: copy owned, T2: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>) -> parser<T0>
{
	seq3(p0, p1, p2, |a0, _a1, _a2| result::Ok(a0))
}

/// seq3 := e0 e1 e2
fn seq3_ret1<T0: copy owned, T1: copy owned, T2: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>) -> parser<T1>
{
	seq3(p0, p1, p2, |_a0, a1, _a2| result::Ok(a1))
}

/// seq3 := e0 e1 e2
fn seq3_ret2<T0: copy owned, T1: copy owned, T2: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>) -> parser<T2>
{
	seq3(p0, p1, p2, |_a0, _a1, a2| result::Ok(a2))
}

/// seq4 := e0 e1 e2 e3
fn seq4_ret0<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<T0>
{
	seq4(p0, p1, p2, p3, |a0, _a1, _a2, _a3| result::Ok(a0))
}

/// seq4 := e0 e1 e2 e3
fn seq4_ret1<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<T1>
{
	seq4(p0, p1, p2, p3, |_a0, a1, _a2, _a3| result::Ok(a1))
}

/// seq4 := e0 e1 e2 e3
fn seq4_ret2<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<T2>
{
	seq4(p0, p1, p2, p3, |_a0, _a1, a2, _a3| result::Ok(a2))
}

/// seq4 := e0 e1 e2 e3
fn seq4_ret3<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<T3>
{
	seq4(p0, p1, p2, p3, |_a0, _a1, _a2, a3| result::Ok(a3))
}

/// s0 := e [ \t\r\n]*
fn s0<T: copy owned>(parser: parser<T>) -> parser<T>
{
	// It would be simpler to write this with scan0, but scan0 is relatively inefficient
	// and s0 is typically called a lot.
	|input: state| {
		do result::chain(parser(input))
		|pass| {
			let mut i = pass.new_state.index;
			let mut line = pass.new_state.line;
			loop
			{
				if input.text[i] == '\r' && input.text[i+1u] == '\n'
				{
					line += 1;
					i += 1u;
				}
				else if input.text[i] == '\n'
				{
					line += 1;
				}
				else if input.text[i] == '\r'
				{
					line += 1;
				}
				else if input.text[i] != ' ' && input.text[i] != '\t'
				{
					break;
				}
				i += 1u;
			}
			
			result::Ok({new_state: {index: i, line: line ,.. pass.new_state}, value: pass.value})
		}
	}
}

/// s1 := e [ \t\r\n]+
fn s1<T: copy owned>(parser: parser<T>) -> parser<T>
{
	|input: state| {
		do result::chain(s0(parser)(input))
		|pass| {
			if option::is_some(str::find_char(" \t\r\n", input.text[pass.new_state.index - 1u]))	// little cheesy, but saves us from adding a helper fn
			{
				result::Ok(pass)
			}
			else
			{
				result::Err({old_state: input, err_state: pass.new_state, mesg: ~"whitespace"})
			}
		}
	}
}


