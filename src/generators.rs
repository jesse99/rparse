#[doc = "Functions used to build parse functions."];

import combinators::*;
import misc::*;
import primitives::*;
import types::*;

#[doc = "Parses with the aid of a pointer to a parser (useful for things like parenthesized expressions)."]
fn forward_ref<T: copy>(parser: @mut parser<T>) -> parser<T>
{
	{|input: state|
		(*parser)(input)
	}
}

#[doc = "Consumes one or more characters matching the predicate.
Returns the matched characters. Note that this does not increment line."]
fn match1(predicate: fn@ (char) -> bool, errMesg: str) -> parser<str>
{
	{|input: state|
		let mut i = input.index;
		while input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		if i > input.index
		{
			let text = str::from_chars(vec::slice(input.text, input.index, i));
			log_ok("match1", input, {new_state: {index: i with input}, value: text})
		}
		else
		{
			log_err("match1", input, {old_state: input, err_state: {index: i with input}, mesg: errMesg})
		}
	}
}

#[doc = "Returns a parser which matches the end of the input.

Typically clients will use the everything method instead of calling this directly."]
fn eot() -> parser<()>
{
	{|input: state|
		if input.text[input.index] == EOT
		{
			log_ok("eot", input, {new_state: {index: input.index + 1u with input}, value: ()})
		}
		else
		{
			log_err("eot", input, {old_state: input, err_state: input, mesg: "EOT"})
		}
	}
}

#[doc = "Parses the text and fails if all the text was not consumed. Leading space is allowed.
Also see parse function.

This is normally the only time leading spaces are parsed and the syntax is a little odd. Use
something like `return(x).space0()` to create space where x is of type T."]
fn everything<T: copy>(parser: parser<T>, space: parser<T>) -> parser<T>
{
	sequence3(space, parser, eot()) {|_a, b, _c| b}
}

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
			log_err(#fmt["text '%s'", s], input, {old_state: input, err_state: {index: j with input}, mesg: #fmt["'%s'", s]})
		}
	}
}

#[doc = "Returns value if input matches s. Also see text."]
fn literal<T: copy>(s: str, value: T) -> parser<T>
{
	{|input: state|
		alt text(s)(input)
		{
			result::ok(pass)
			{
				log_ok("literal", input, {new_state: pass.new_state, value: value})
			}
			result::err(failure)
			{
				log_err(#fmt["literal '%s'", s], input, failure)
			}
		}
	}
}

#[doc = "integer := [+-]? [0-9]+"]
fn integer() -> parser<int>
{
	let digits = match1(is_digit, "digits").then({|s| return(option::get(int::from_str(s)))});
	let case1 = text("+")._then(digits);
	let case2 = sequence2(text("-"), digits, {|_o, v| -v});
	let case3 = digits;
	alternative([case1, case2, case3])
}

#[doc = "identifier := [a-zA-Z_] [a-zA-Z0-9_]*"]
fn identifier() -> parser<str>
{
	let prefix = match1(is_identifier_prefix, "identifier");
	let suffix = match1(is_identifier_suffix, "identifier").repeat0();
	prefix.then({|p| suffix.then({|s| return(p + str::connect(s, ""))})})
}
