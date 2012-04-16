#[doc = "Functions used to build parse function using parse functions."];

import basis::*;
import types::*;

impl std_combinators<T: copy> for parser<T>
{
	#[doc = "If everything is successful then parser2 is called (and the value from self
	is ignored). If self fails parser2 is not called. Also see then."]
	fn _then<T: copy, U: copy>(parser2: parser<U>) -> parser<U>
	{
		{|input: state|
			result::chain(self(input))
			{|pass|
				result::chain_err(parser2(pass.new_state))
				{|failure|
					log_err("_then", input, {old_state: input with failure})
				}
			}
		}
	}
	
	#[doc = "space0 := e [ \t\r\n]*"]
	fn space0<T: copy>() -> parser<T>
	{
		{|input: state|
			result::chain(self(input))
			{|pass|
				let mut i = pass.new_state.index;
				let mut line = pass.new_state.line;
				while true
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
				
				log_ok("space0", input, {new_state: {index: i, line: line with pass.new_state}, value: pass.value})
			}
		}
	}
	
	#[doc = "space1 := e [ \t\r\n]+"]
	fn space1<T: copy>() -> parser<T>
	{
		{|input: state|
			result::chain(self.space0()(input))
			{|pass|
				if option::is_some(str::find_char(" \t\r\n", input.text[pass.new_state.index - 1u]))	// little cheesy, but saves us from adding a helper fn
				{
					log_ok("space1", input, pass)
				}
				else
				{
					log_err("space1", input, {old_state: input, err_state: pass.new_state, mesg: "whitespace"})
				}
			}
		}
	}
	
	#[doc = "optional := e?"]
	fn optional<T: copy>(missing: T) -> parser<T>
	{
		{|input: state|
			result::chain_err(self(input))
			{|_failure|
				log_ok("optional", input, {new_state: input, value: missing})
			}
		}
	}
	
	#[doc = "repeat0 := e*
	
	Values for each parsed e are returned."]
	fn repeat0<T: copy>() -> parser<[T]>
	{
		{|input: state|
			let mut output = input;
			let mut values = [];
			loop
			{
				alt self(output)
				{
					result::ok(pass)
					{
						assert pass.new_state.index > output.index;	// must make progress to ensure loop termination
						output = pass.new_state;
						vec::push(values, pass.value);
					}
					result::err(failure)
					{
						break;
					}
				}
			}
			log_ok("repeat0", input, {new_state: output, value: values})
		}
	}
	
	#[doc = "repeat1 := e+
	
	Values for each parsed e are returned."]
	fn repeat1<T: copy>(err_mesg: str) -> parser<[T]>
	{
		{|input: state|
			let pass = result::get(self.repeat0()(input));
			if pass.new_state.index > input.index
			{
				log_ok("repeat1", input, pass)
			}
			else
			{
				log_err("repeat1", input, {old_state: input, err_state: pass.new_state, mesg: err_mesg})
			}
		}
	}
	
	#[doc = "list := e (sep e)*
	
	Values for each parsed e are returned."]
	fn list<T: copy, U: copy>(sep: parser<U>) -> parser<[T]>
	{
		let term = sep._then(self).repeat0();
		
		{|input: state|
			result::chain(self(input))
			{|pass|
				alt term(pass.new_state)
				{
					result::ok(pass2)
					{
						log_ok("list", input, {value: [pass.value] + pass2.value with pass2})
					}
					result::err(failure)
					{
						log_err("list", input, {old_state: input with failure})
					}
				}
			}
		}
	}
	
	// chain_suffix := (op e)*
	#[doc(hidden)]
	fn chain_suffix<T: copy, U: copy>(op: parser<U>) -> parser<[(U, T)]>
	{
		let q = op.then({|operator| self.then({|value| return((operator, value))})});
		q.repeat0()
	}
	
	#[doc = "chainl1 := e (op e)*
	
	Left associative binary operator. eval is called for each parsed op."]
	fn chainl1<T: copy, U: copy>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
	{
		{|input: state|
			result::chain(self(input))
			{|pass|
				alt self.chain_suffix(op)(pass.new_state)
				{
					result::ok(pass2)
					{
						let value = vec::foldl(pass.value, pass2.value, {|lhs, rhs| eval(lhs, tuple::first(rhs), tuple::second(rhs))});
						log_ok("chainl1", input, {new_state: pass2.new_state, value: value})
					}
					result::err(failure)
					{
						log_err("chainl1", input, {old_state: input with failure})
					}
				}
			}
		}
	}
	
	#[doc = "chainr1 := e (op e)*
	
	Right associative binary operator. eval is called for each parsed op."]
	fn chainr1<T: copy, U: copy>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
	{
		{|input: state|
			result::chain(self(input))
			{|pass|
				alt self.chain_suffix(op)(pass.new_state)
				{
					result::ok(pass2)
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
							let parsers = [e1] + vec::slice(parsers, 0u, vec::len(parsers) - 1u);
							
							// [(e1 op1), (e2 op2)] and e3
							let terms = vec::zip(parsers, ops);
							
							let value = vec::foldr(terms, e3, {|lhs, rhs| eval(tuple::first(lhs), tuple::second(lhs), rhs)});
							log_ok("chainr1", input, {new_state: pass2.new_state, value: value})
						}
						else
						{
							log_ok("chainr1", input, {new_state: pass2.new_state, value: pass.value})
						}
					}
					result::err(failure)
					{
						log_err("chainr1", input, {old_state: input with failure})
					}
				}
			}
		}
	}
}

#[doc = "alternative := e0 | e1 | …

This is a version of or that is nicer to use when there are more than two alternatives."]
fn alternative<T: copy>(parsers: [parser<T>]) -> parser<T>
{
	// A recursive algorithm would be a lot simpler, but it's not clear how that could
	// produce good error messages.
	assert vec::is_not_empty(parsers);
	
	{|input: state|
		let mut result: option<status<T>> = none;
		let mut errors = [];
		let mut i = 0u;
		while i < vec::len(parsers) && option::is_none(result)
		{
			alt parsers[i](input)
			{
				result::ok(pass)
				{
					result = option::some(log_ok("alternative", input, pass));
				}
				result::err(failure)
				{
					vec::push(errors, failure.mesg);
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
			let mesg = str::connect(errors, " or ");
			log_err("alternative", input, {old_state: input, err_state: input, mesg: mesg})
		}
	}
}

#[doc = "sequence2 := e0 e1

If the parses succeed eval is called with the value from each parse. This is a version 
of then that is often simpler to use."]
fn sequence2<T0: copy, T1: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, eval: fn@ (T0, T1) -> R) -> parser<R>
{
	parser0.then()
	{|a0|
		parser1.then({|a1| return(eval(a0, a1))})
	}
}

#[doc = "sequence3 := e0 e1 e2

If the parses succeed eval is called with the value from each parse. This is a version 
of then that is often simpler to use."]
fn sequence3<T0: copy, T1: copy, T2: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, eval: fn@ (T0, T1, T2) -> R) -> parser<R>
{
	parser0.then()
	{|a0|
		parser1.then()
		{|a1|
			parser2.then({|a2| return(eval(a0, a1, a2))})
		}
	}
}

#[doc = "sequence4 := e0 e1 e2 e3

If the parses succeed eval is called with the value from each parse. This is a version 
of then that is often simpler to use."]
fn sequence4<T0: copy, T1: copy, T2: copy, T3: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, eval: fn@ (T0, T1, T2, T3) -> R) -> parser<R>
{
	parser0.then()
	{|a0|
		parser1.then()
		{|a1|
			parser2.then()
			{|a2|
				parser3.then({|a3| return(eval(a0, a1, a2, a3))})
			}
		}
	}
}
