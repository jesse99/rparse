#[doc = "Functions used to build parse functions."];

import basis::*;
import types::*;

#[doc = "Returns s if input matches s. Also see literal."]
fn text(s: str) -> parser<str>
{
	{|input: state|
		let mut i = 0u;
		let mut j = input.index;
		while i < str::len(s)
		{
			let {ch, next} = str::char_range_at(s, i);
			if ch == input.text[j]
			{
				i = next;
				j += 1u;
			}
			else
			{
				break;
			}
		}
		
		if i == str::len(s)
		{
			let text = str::from_chars(vec::slice(input.text, input.index, j));
			log_ok("text", input, {new_state: {index: j with input}, value: text})
		}
		else
		{
			log_err("text", input, {old_state: input, err_state: {index: j with input}, mesg: #fmt["'%s'", s]})
		}
	}
}

#[doc = "Returns value if input matches s. Also see text."]
fn literal<T: copy>(s: str, value: T) -> parser<T>
{
	{|input: state|
		chain(text(s)(input))
		{|pass|
			log_ok("literal", input, {new_state: pass.new_state, value: value})
		}
	}
}

/*
#[doc = "integer := [+-] [0-9]+"]
fn integer() -> parser<int>
{
	{|input: state|
		chain(text(s)(input))
		{|pass|
			log_ok("literal", input, {new_state: pass.new_state, value: value})
		}
	}
}
*/

// identifier
