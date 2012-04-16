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
}

#[doc = "sequence := e0 e1...

If the parses succeed eval is called with the value from each parse. This is a version 
of then that is simpler to use with more than two parsers (assuming that they all 
return the same type)."]
fn sequence<T: copy, U: copy>(parsers: [parser<T>], eval: fn@ ([T]) -> U) -> parser<U>
{
	{|input: state|
		let mut output = input;
		let mut result = result::err({old_state: input, err_state: input, mesg: "dummy"});
		let mut values = [];
		vec::reserve(values, vec::len(parsers));
		
		let mut i = 0u;
		while i < vec::len(parsers)
		{
			alt parsers[i](output)
			{
				result::ok(pass)
				{
					output = pass.new_state;
					vec::push(values, pass.value);
					i += 1u;
				}
				result::err(failure)
				{
					result = log_err("sequence", input, {old_state: input with failure});
					break;
				}
			}
		}
		
		if i == vec::len(parsers)
		{
			result = log_ok("sequence", input, {new_state: output, value: eval(values)})
		}
		result
	}
}
