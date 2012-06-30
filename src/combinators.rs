#[doc = "Functions and methods used to compose parsers.

Note that these functions and methods don't actually consume input (although 
the parsers they are invoked with normally will)."];

// chain_suffix := (op e)*
#[doc(hidden)]
fn chain_suffix<T: copy, U: copy>(parser: parser<T>, op: parser<U>) -> parser<[(U, T)]/~>
{
	let q = op.thene({|operator| parser.thene({|value| return((operator, value))})});
	q.r0()
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
						let parsers = [e1]/~ + vec::slice(parsers, 0u, vec::len(parsers) - 1u);
						
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

#[doc = "Parses with the aid of a pointer to a parser (useful for things like parenthesized expressions)."]
fn forward_ref<T: copy>(parser: @mut parser<T>) -> parser<T>
{
	{|input: state|
		(*parser)(input)
	}
}
#[doc = "list := e (sep e)*

Values for each parsed e are returned."]
fn list<T: copy, U: copy>(parser: parser<T>, sep: parser<U>) -> parser<[T]/~>
{
	let term = sep.then(parser).r0();
	
	{|input: state|
		result::chain(parser(input))
		{|pass|
			alt term(pass.new_state)
			{
				result::ok(pass2)
				{
					log_ok("list", input, {value: [pass.value]/~ + pass2.value with pass2})
				}
				result::err(failure)
				{
					log_err("list", input, {old_state: input with failure})
				}
			}
		}
	}
}

#[doc = "optional := e?"]
fn optional<T: copy>(parser: parser<T>) -> parser<option<T>>
{
	{|input: state|
		alt parser(input)
		{
			result::ok(pass)
			{
				log_ok("optional", input, {new_state: pass.new_state, value: option::some(pass.value)})
			}
			result::err(_failure)
			{
				log_ok("optional", input, {new_state: input, value: option::none})
			}
		}
	}
}

// When using tag it can be useful to use empty messages for interior parsers
// so we need to handle that case.
fn or_mesg(mesg1: str, mesg2: str) -> str
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
		""
	}
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
						log_err("or", input, {mesg: or_mesg(failure1.mesg, mesg2) with failure2})
					}
					else
					{
						log_err("or", input, {mesg: or_mesg(failure1.mesg, failure2.mesg) with failure2})
					}
				}
			}
		}
	}
}

#[doc = "or_v := e0 | e1 | …

This is a version of or that is nicer to use when there are more than two alternatives."]
fn or_v<T: copy>(parsers: [parser<T>]/~) -> parser<T>
{
	// A recursive algorithm would be a lot simpler, but it's not clear how that could
	// produce good error messages.
	assert vec::is_not_empty(parsers);
	
	{|input: state|
		let mut result: option<status<T>> = none;
		let mut errors = []/~;
		let mut max_index = uint::max_value;
		let mut i = 0u;
		while i < vec::len(parsers) && option::is_none(result)
		{
			alt parsers[i](input)
			{
				result::ok(pass)
				{
					result = option::some(log_ok("or_v", input, pass));
				}
				result::err(failure)
				{
					if failure.err_state.index > max_index || max_index == uint::max_value
					{
						errors = [failure.mesg]/~;
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
			let errs = vec::filter(errors) {|s| str::is_not_empty(s)};
			let mesg = str::connect(errs, " or ");
			log_err("or_v", input, {old_state: input, err_state: {index: max_index with input}, mesg: mesg})
		}
	}
}

#[doc = "Succeeds if parser matches input n to m times (inclusive)."]
fn r<T: copy>(parser: parser<T>, n: uint, m: uint) -> parser<[T]/~>
{
	{|input: state|
		let mut output = input;
		let mut values = []/~;
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
		
		let count = vec::len(values);
		if n <= count && count <= m
		{
			log_ok("r", input, {new_state: output, value: values})
		}
		else
		{
			log_err("r", input, {old_state: input, err_state: output, mesg: ""})
		}
	}
}

#[doc = "r0 := e*

Values for each parsed e are returned."]
fn r0<T: copy>(parser: parser<T>) -> parser<[T]/~>
{
	r(parser, 0u, uint::max_value)
}

#[doc = "r1 := e+

Values for each parsed e are returned."]
fn r1<T: copy>(parser: parser<T>) -> parser<[T]/~>
{
	r(parser, 1u, uint::max_value)
}

#[doc = "seq2 := e0 e1"]
fn seq2<T0: copy, T1: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, eval: fn@ (T0, T1) -> result::result<R, str>) -> parser<R>
{
	parser0.thene() {|a0|
	parser1.thene() {|a1|
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

#[doc = "seq3 := e0 e1 e2"]
fn seq3<T0: copy, T1: copy, T2: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, eval: fn@ (T0, T1, T2) -> result::result<R, str>) -> parser<R>
{
	parser0.thene() {|a0|
	parser1.thene() {|a1|
	parser2.thene() {|a2|
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

#[doc = "seq4 := e0 e1 e2 e3"]
fn seq4<T0: copy, T1: copy, T2: copy, T3: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, eval: fn@ (T0, T1, T2, T3) -> result::result<R, str>) -> parser<R>
{
	parser0.thene() {|a0|
	parser1.thene() {|a1|
	parser2.thene() {|a2|
	parser3.thene() {|a3|
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

#[doc = "seq5 := e0 e1 e2 e3 e4"]
fn seq5<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, eval: fn@ (T0, T1, T2, T3, T4) -> result::result<R, str>) -> parser<R>
{
	parser0.thene() {|a0|
	parser1.thene() {|a1|
	parser2.thene() {|a2|
	parser3.thene() {|a3|
	parser4.thene() {|a4|
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

#[doc = "seq6 := e0 e1 e2 e3 e4 e5"]
fn seq6<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, T5: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, parser5: parser<T5>, eval: fn@ (T0, T1, T2, T3, T4, T5) -> result::result<R, str>) -> parser<R>
{
	parser0.thene() {|a0|
	parser1.thene() {|a1|
	parser2.thene() {|a2|
	parser3.thene() {|a3|
	parser4.thene() {|a4|
	parser5.thene() {|a5|
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

#[doc = "seq7 := e0 e1 e2 e3 e4 e5 e6"]
fn seq7<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, T5: copy, T6: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, parser5: parser<T5>, parser6: parser<T6>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6) -> result::result<R, str>) -> parser<R>
{
	parser0.thene() {|a0|
	parser1.thene() {|a1|
	parser2.thene() {|a2|
	parser3.thene() {|a3|
	parser4.thene() {|a4|
	parser5.thene() {|a5|
	parser6.thene() {|a6|
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

#[doc = "seq8 := e0 e1 e2 e3 e4 e5 e6 e7"]
fn seq8<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, T5: copy, T6: copy, T7: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, parser5: parser<T5>, parser6: parser<T6>, parser7: parser<T7>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6, T7) -> result::result<R, str>) -> parser<R>
{
	parser0.thene() {|a0|
	parser1.thene() {|a1|
	parser2.thene() {|a2|
	parser3.thene() {|a3|
	parser4.thene() {|a4|
	parser5.thene() {|a5|
	parser6.thene() {|a6|
	parser7.thene() {|a7|
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

#[doc = "seq9 := e0 e1 e2 e3 e4 e5 e6 e7 e8"]
fn seq9<T0: copy, T1: copy, T2: copy, T3: copy, T4: copy, T5: copy, T6: copy, T7: copy, T8: copy, R: copy>
	(parser0: parser<T0>, parser1: parser<T1>, parser2: parser<T2>, parser3: parser<T3>, parser4: parser<T4>, parser5: parser<T5>, parser6: parser<T6>, parser7: parser<T7>, parser8: parser<T8>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6, T7, T8) -> result::result<R, str>) -> parser<R>
{
	parser0.thene() {|a0|
	parser1.thene() {|a1|
	parser2.thene() {|a2|
	parser3.thene() {|a3|
	parser4.thene() {|a4|
	parser5.thene() {|a5|
	parser6.thene() {|a6|
	parser7.thene() {|a7|
	parser8.thene() {|a8|
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

#[doc = "seq2 := e0 e1"]
fn seq2_ret0<T0: copy, T1: copy>(p0: parser<T0>, p1: parser<T1>) -> parser<T0>
{
	seq2(p0, p1)
		{|a0, _a1| result::ok(a0)}
}

#[doc = "seq2 := e0 e1"]
fn seq2_ret1<T0: copy, T1: copy>(p0: parser<T0>, p1: parser<T1>) -> parser<T1>
{
	seq2(p0, p1)
		{|_a0, a1| result::ok(a1)}
}

#[doc = "seq3 := e0 e1 e2"]
fn seq3_ret0<T0: copy, T1: copy, T2: copy>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>) -> parser<T0>
{
	seq3(p0, p1, p2)
		{|a0, _a1, _a2| result::ok(a0)}
}

#[doc = "seq3 := e0 e1 e2"]
fn seq3_ret1<T0: copy, T1: copy, T2: copy>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>) -> parser<T1>
{
	seq3(p0, p1, p2)
		{|_a0, a1, _a2| result::ok(a1)}
}

#[doc = "seq3 := e0 e1 e2"]
fn seq3_ret2<T0: copy, T1: copy, T2: copy>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>) -> parser<T2>
{
	seq3(p0, p1, p2)
		{|_a0, _a1, a2| result::ok(a2)}
}

#[doc = "seq4 := e0 e1 e2 e3"]
fn seq4_ret0<T0: copy, T1: copy, T2: copy, T3: copy>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<T0>
{
	seq4(p0, p1, p2, p3)
		{|a0, _a1, _a2, _a3| result::ok(a0)}
}

#[doc = "seq4 := e0 e1 e2 e3"]
fn seq4_ret1<T0: copy, T1: copy, T2: copy, T3: copy>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<T1>
{
	seq4(p0, p1, p2, p3)
		{|_a0, a1, _a2, _a3| result::ok(a1)}
}

#[doc = "seq4 := e0 e1 e2 e3"]
fn seq4_ret2<T0: copy, T1: copy, T2: copy, T3: copy>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<T2>
{
	seq4(p0, p1, p2, p3)
		{|_a0, _a1, a2, _a3| result::ok(a2)}
}

#[doc = "seq4 := e0 e1 e2 e3"]
fn seq4_ret3<T0: copy, T1: copy, T2: copy, T3: copy>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<T3>
{
	seq4(p0, p1, p2, p3)
		{|_a0, _a1, _a2, a3| result::ok(a3)}
}

#[doc = "s0 := e [ \t\r\n]*"]
fn s0<T: copy>(parser: parser<T>) -> parser<T>
{
	// It would be simpler to write this with scan0, but scan0 is relatively inefficient
	// and s0 is typically called a lot.
	{|input: state|
		result::chain(parser(input))
		{|pass|
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
			
			log_ok("s0", input, {new_state: {index: i, line: line with pass.new_state}, value: pass.value})
		}
	}
}

#[doc = "s1 := e [ \t\r\n]+"]
fn s1<T: copy>(parser: parser<T>) -> parser<T>
{
	{|input: state|
		result::chain(s0(parser)(input))
		{|pass|
			if option::is_some(str::find_char(" \t\r\n", input.text[pass.new_state.index - 1u]))	// little cheesy, but saves us from adding a helper fn
			{
				log_ok("s1", input, pass)
			}
			else
			{
				log_err("s1", input, {old_state: input, err_state: pass.new_state, mesg: "Expected whitespace"})
			}
		}
	}
}


#[doc = "Adds custom text to rules as they match or fail to match."]
fn annotate<T: copy>(parser: parser<T>, text: str) -> parser<T>
{
	{|input: state|
		alt parser(input)
		{
			result::ok(pass)
			{
				log_ok(text, input, pass)
			}
			result::err(failure)
			{
				log_err(text, input, failure)
			}
		}
	}
}

#[doc = "If label is not empty then it is used if the parser completely failed to parse or if its error
message was empty. Otherwise it suppresses errors from the parser (in favor of a later tag function).

Should be of the form \"Expected foo\"."]
fn tag<T: copy>(parser: parser<T>, label: str) -> parser<T>
{
	{|input: state|
		result::chain_err(parser(input))
		{|failure|
			if str::is_empty(label)
			{
				log_err("tag", input, {mesg: "" with failure})
			}
			else if failure.err_state.index == input.index || str::is_empty(failure.mesg)
			{
				log_err("tag", input, {mesg: label with failure})
			}
			else
			{
				// If we managed to parse something then it is usually better to
				// use that error message. (If that's not what you want then use
				// empty strings there).
				log_err("tag", input, failure)
			}
		}
	}
}

#[doc = "If parser1 is successful is successful then parser2 is called (and the value from parser1
is ignored). If parser1 fails parser2 is not called."]
fn then<T: copy, U: copy>(parser1: parser<T>, parser2: parser<U>) -> parser<U>
{
	{|input: state|
		result::chain(parser1(input))
		{|pass|
			result::chain_err(parser2(pass.new_state))
			{|failure|
				log_err("then", input, {old_state: input with failure})
			}
		}
	}
}

#[doc = "If parser is successful then the function returned by eval is called
with parser's result. If parser fails eval is not called.

Often used to translate parsed values: `p().thene({|pvalue| return(2*pvalue)})`"]
fn thene<T: copy, U: copy>(parser: parser<T>, eval: fn@ (T) -> parser<U>) -> parser<U>
{
	{|input: state|
		result::chain(parser(input))
		{|pass|
			result::chain_err(eval(pass.value)(pass.new_state))
			{|failure|
				log_err("thene", input, {old_state: input with failure})
			}
		}
	}
}

