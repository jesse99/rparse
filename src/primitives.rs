#[doc = "Functions and methods used to construct and compose parsers.

Note that these functions and methods don't actually consume input (although 
the parsers they are invoked with normally will)."];

import misc::*;
import types::*;

#[doc = "Returns a parser which always fails."]
fn fails<T: copy>(mesg: str) -> parser<T>
{
	{|input: state|
		log_err("fails", input, {old_state: input, err_state: input, mesg: mesg})}
}

#[doc = "Returns a parser which always succeeds, but does not consume any input."]
fn return<T: copy>(value: T) -> parser<T>
{
	{|input: state|
		log_ok("return", input, {new_state: input, value: value})}
}

#[doc = "If parser is successful then the function returned by eval is called
with parser's result. If parser fails eval is not called.

Often used to translate parsed values: `p().then({|v| return(blah::from_whatever(v))})`"]
fn then<T: copy, U: copy>(parser: parser<T>, eval: fn@ (T) -> parser<U>) -> parser<U>
{
	{|input: state|
		result::chain(parser(input))
		{|pass|
			result::chain_err(eval(pass.value)(pass.new_state))
			{|failure|
				log_err("then", input, {old_state: input with failure})
			}
		}
	}
}

#[doc = "If parser1 is successful is successful then parser2 is called (and the value from parser1
is ignored). If parser1 fails parser2 is not called."]
fn _then<T: copy, U: copy>(parser1: parser<T>, parser2: parser<U>) -> parser<U>
{
	{|input: state|
		result::chain(parser1(input))
		{|pass|
			result::chain_err(parser2(pass.new_state))
			{|failure|
				log_err("_then", input, {old_state: input with failure})
			}
		}
	}
}

#[doc = "sequence2 := e0 e1"]
fn sequence2<T0: copy, T1: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, eval: fn@ (T0, T1) -> result::result<R, str>) -> parser<R>
{
	parser0.then() {|a0|
	parser1.then() {|a1|
		alt eval(a0, a1)
		{
			result::ok(value)
			{
				return(value)
			}
			result::err(mesg)
			{
				fails(mesg)
			}
		}
	}}
}

#[doc = "sequence3 := e0 e1 e2"]
fn sequence3<T0: copy, T1: copy, T2: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, eval: fn@ (T0, T1, T2) -> result::result<R, str>) -> parser<R>
{
	parser0.then() {|a0|
	parser1.then() {|a1|
	parser2.then() {|a2|
		alt eval(a0, a1, a2)
		{
			result::ok(value)
			{
				return(value)
			}
			result::err(mesg)
			{
				fails(mesg)
			}
		}
	}}}
}

#[doc = "sequence4 := e0 e1 e2 e3"]
fn sequence4<T0: copy, T1: copy, T2: copy, T3: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, eval: fn@ (T0, T1, T2, T3) -> result::result<R, str>) -> parser<R>
{
	parser0.then() {|a0|
	parser1.then() {|a1|
	parser2.then() {|a2|
	parser3.then() {|a3|
		alt eval(a0, a1, a2, a3)
		{
			result::ok(value)
			{
				return(value)
			}
			result::err(mesg)
			{
				fails(mesg)
			}
		}
	}}}}
}

#[doc = "sequence5 := e0 e1 e2 e3 e4"]
fn sequence5<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, eval: fn@ (T0, T1, T2, T3, T4) -> result::result<R, str>) -> parser<R>
{
	parser0.then() {|a0|
	parser1.then() {|a1|
	parser2.then() {|a2|
	parser3.then() {|a3|
	parser4.then() {|a4|
		alt eval(a0, a1, a2, a3, a4)
		{
			result::ok(value)
			{
				return(value)
			}
			result::err(mesg)
			{
				fails(mesg)
			}
		}
	}}}}}
}

#[doc = "sequence6 := e0 e1 e2 e3 e4 e5"]
fn sequence6<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, T5: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, parser5: parser<T5>, eval: fn@ (T0, T1, T2, T3, T4, T5) -> result::result<R, str>) -> parser<R>
{
	parser0.then() {|a0|
	parser1.then() {|a1|
	parser2.then() {|a2|
	parser3.then() {|a3|
	parser4.then() {|a4|
	parser5.then() {|a5|
		alt eval(a0, a1, a2, a3, a4, a5)
		{
			result::ok(value)
			{
				return(value)
			}
			result::err(mesg)
			{
				fails(mesg)
			}
		}
	}}}}}}
}

#[doc = "sequence7 := e0 e1 e2 e3 e4 e5 e6"]
fn sequence7<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, T5: copy, T6: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, parser5: parser<T5>, parser6: parser<T6>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6) -> result::result<R, str>) -> parser<R>
{
	parser0.then() {|a0|
	parser1.then() {|a1|
	parser2.then() {|a2|
	parser3.then() {|a3|
	parser4.then() {|a4|
	parser5.then() {|a5|
	parser6.then() {|a6|
		alt eval(a0, a1, a2, a3, a4, a5, a6)
		{
			result::ok(value)
			{
				return(value)
			}
			result::err(mesg)
			{
				fails(mesg)
			}
		}
	}}}}}}}
}

#[doc = "sequence8 := e0 e1 e2 e3 e4 e5 e6 e7"]
fn sequence8<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, T5: copy, T6: copy, T7: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, parser5: parser<T5>, parser6: parser<T6>, parser7: parser<T7>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6, T7) -> result::result<R, str>) -> parser<R>
{
	parser0.then() {|a0|
	parser1.then() {|a1|
	parser2.then() {|a2|
	parser3.then() {|a3|
	parser4.then() {|a4|
	parser5.then() {|a5|
	parser6.then() {|a6|
	parser7.then() {|a7|
		alt eval(a0, a1, a2, a3, a4, a5, a6, a7)
		{
			result::ok(value)
			{
				return(value)
			}
			result::err(mesg)
			{
				fails(mesg)
			}
		}
	}}}}}}}}
}

#[doc = "sequence9 := e0 e1 e2 e3 e4 e5 e6 e7 e8"]
fn sequence9<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, T5: copy, T6: copy, T7: copy, T8: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, parser5: parser<T5>, parser6: parser<T6>, parser7: parser<T7>, parser8: parser<T8>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6, T7, T8) -> result::result<R, str>) -> parser<R>
{
	parser0.then() {|a0|
	parser1.then() {|a1|
	parser2.then() {|a2|
	parser3.then() {|a3|
	parser4.then() {|a4|
	parser5.then() {|a5|
	parser6.then() {|a6|
	parser7.then() {|a7|
	parser8.then() {|a8|
		alt eval(a0, a1, a2, a3, a4, a5, a6, a7, a8)
		{
			result::ok(value)
			{
				return(value)
			}
			result::err(mesg)
			{
				fails(mesg)
			}
		}
	}}}}}}}}}
}

#[doc = "Returns a parser which first tries parser1, and if that fails, parser2."]
fn or<T: copy>(parser1: parser<T>, parser2: parser<T>) -> parser<T>
{
	{|input: state|
		result::chain_err(parser1(input))
		{|failure1|
			result::chain_err(parser2(input))
			{|failure2|
				if failure1.err_state.index > failure2.err_state.index
				{
					log_err("or", input, failure1)
				}
				else if failure1.err_state.index < failure2.err_state.index
				{
					log_err("or", input, failure2)
				}
				else
				{
					if str::starts_with(failure2.mesg, "Expected ")
					{
						let mesg2 = str::slice(failure2.mesg, str::len("Expected "), str::len(failure2.mesg));
						log_err("or", input, {mesg: failure1.mesg + " or " + mesg2 with failure2})
					}
					else
					{
						log_err("or", input, {mesg: failure1.mesg + " or " + failure2.mesg with failure2})
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
		let mut max_index = uint::max_value;
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
					if failure.err_state.index > max_index || max_index == uint::max_value
					{
						errors = [failure.mesg];
						max_index = failure.err_state.index;
					}
					else if failure.err_state.index == max_index
					{
						if str::starts_with(failure.mesg, "Expected ")
						{
							let mesg = str::slice(failure.mesg, str::len("Expected "), str::len(failure.mesg));
							vec::push(errors, mesg);
						}
						else
						{
							vec::push(errors, failure.mesg);
						}
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
			let mesg = str::connect(errors, " or ");
			log_err("alternative", input, {old_state: input, err_state: {index: max_index with input}, mesg: mesg})
		}
	}
}

#[doc = "optional := e?"]
fn optional<T: copy>(parser: parser<T>, missing: T) -> parser<T>
{
	{|input: state|
		result::chain_err(parser(input))
		{|_failure|
			log_ok("optional", input, {new_state: input, value: missing})
		}
	}
}

#[doc = "repeat0 := e*

Values for each parsed e are returned."]
fn repeat0<T: copy>(parser: parser<T>) -> parser<[T]>
{
	{|input: state|
		let mut output = input;
		let mut values = [];
		loop
		{
			alt parser(output)
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
fn repeat1<T: copy>(parser: parser<T>, err_mesg: str) -> parser<[T]>
{
	{|input: state|
		let pass = result::get(parser.repeat0()(input));
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
fn list<T: copy, U: copy>(parser: parser<T>, sep: parser<U>) -> parser<[T]>
{
	let term = sep._then(parser).repeat0();
	
	{|input: state|
		result::chain(parser(input))
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
fn chain_suffix<T: copy, U: copy>(parser: parser<T>, op: parser<U>) -> parser<[(U, T)]>
{
	let q = op.then({|operator| parser.then({|value| return((operator, value))})});
	q.repeat0()
}

#[doc = "chainl1 := e (op e)*

Left associative binary operator. eval is called for each parsed op."]
fn chainl1<T: copy, U: copy>(parser: parser<T>, op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
{
	{|input: state|
		result::chain(parser(input))
		{|pass|
			alt parser.chain_suffix(op)(pass.new_state)
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
fn chainr1<T: copy, U: copy>(parser: parser<T>, op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
{
	{|input: state|
		result::chain(parser(input))
		{|pass|
			alt parser.chain_suffix(op)(pass.new_state)
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
	
#[doc = "If parser completely fails to parse then use label as the error message."]
fn tag<T: copy>(parser: parser<T>, label: str) -> parser<T>
{
	{|input: state|
		result::chain_err(parser(input))
		{|failure|
			if failure.err_state.index == input.index
			{
				log_err("tag", input, {mesg: label with failure})
			}
			else
			{
				// If we managed to parse something then it is usually better to
				// use that error message.
				log_err("tag", input, failure)
			}
		}
	}
}

#[doc = "Parses with the aid of a pointer to a parser (useful for things like parenthesized expressions)."]
fn forward_ref<T: copy>(parser: @mut parser<T>) -> parser<T>
{
	{|input: state|
		(*parser)(input)
	}
}

#[doc = "Return type of parse function."]
type parse_status<T: copy> = result::result<T, parse_failed>;

#[doc = "Returned by parse function on error. Line and col are both 1-based."]
type parse_failed = {file: str, line: uint, col: uint, mesg: str};

#[doc = "Uses parser to parse text. Also see everything function."]
fn parse<T: copy>(parser: parser<T>, file: str, text: str) -> parse_status<T>
{
	let chars = chars_with_eot(text);
	let input = {file: file, text: chars, index: 0u, line: 1};
	alt parser(input)
	{
		result::ok(pass)
		{
			result::ok(pass.value)
		}
		result::err(failure)
		{
			let col = get_col(chars, failure.err_state.index);
			result::err({file: failure.old_state.file, line: failure.err_state.line as uint, col: col, mesg: failure.mesg})
		}
	}
}

#[doc = "These work the same as the functions of the same name, but tend
to make the code look a bit better."]
impl primitive_methods<T: copy> for parser<T>
{
	fn then<T: copy, U: copy>(eval: fn@ (T) -> parser<U>) -> parser<U>
	{
		then(self, eval)
	}
	
	fn _then<T: copy, U: copy>(parser2: parser<U>) -> parser<U>
	{
		_then(self, parser2)
	}
	
	fn or<T: copy>(parser2: parser<T>) -> parser<T>
	{
		or(self, parser2)
	}
	
	fn optional<T: copy>(missing: T) -> parser<T>
	{
		optional(self, missing)
	}
	
	fn repeat0<T: copy>() -> parser<[T]>
	{
		repeat0(self)
	}
	
	fn repeat1<T: copy>(err_mesg: str) -> parser<[T]>
	{
		repeat1(self, err_mesg)
	}
	
	fn list<T: copy, U: copy>(sep: parser<U>) -> parser<[T]>
	{
		list(self, sep)
	}
	
	fn chain_suffix<T: copy, U: copy>(op: parser<U>) -> parser<[(U, T)]>
	{
		chain_suffix(self, op)
	}
	
	fn chainl1<T: copy, U: copy>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
	{
		chainl1(self, op, eval)
	}
	
	fn chainr1<T: copy, U: copy>(op: parser<U>, eval: fn@ (T, U, T) -> T) -> parser<T>
	{
		chainr1(self, op, eval)
	}
	
	fn tag<T: copy>(label: str) -> parser<T>
	{
		tag(self, label)
	}
	
	fn parse(file: str, text: str) -> parse_status<T>
	{
		parse(self, file, text)
	}
}
