#[doc = "Functions used to create parse functions that actually consume input."];

import misc::*;
import parsers::*;
import primitives::*;
import types::*;

#[doc = "space0 := e [ \t\r\n]*"]
fn space0<T: copy>(parser: parser<T>) -> parser<T>
{
	// It would be simpler to write this with scan0, but scan0 is relatively inefficient
	// and space0 is typically called a lot.
	{|input: state|
		result::chain(parser(input))
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
fn space1<T: copy>(parser: parser<T>) -> parser<T>
{
	{|input: state|
		result::chain(space0(parser)(input))
		{|pass|
			if option::is_some(str::find_char(" \t\r\n", input.text[pass.new_state.index - 1u]))	// little cheesy, but saves us from adding a helper fn
			{
				log_ok("space1", input, pass)
			}
			else
			{
				log_err("space1", input, {old_state: input, err_state: pass.new_state, mesg: "Expected whitespace"})
			}
		}
	}
}

#[doc = "Consumes one or more characters matching the predicate.
Returns the matched characters. 

Note that this does not increment line."]
fn match1(predicate: fn@ (char) -> bool, err_mesg: str) -> parser<str>
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
			log_err("match1", input, {old_state: input, err_state: {index: i with input}, mesg: err_mesg})
		}
	}
}

#[doc = "Calls fun with an index into the characters to be parsed until it returns zero characters.
Returns the matched characters. 

This does increment line."]
fn scan0(fun: fn@ ([char], uint) -> uint) -> parser<str>
{
	{|input: state|
		let mut i = input.index;
		let mut line = input.line;
		let mut result = result::err({old_state: input, err_state: input, mesg: "dummy"});
		while result::is_failure(result)
		{
			let count = fun(input.text, i);
			if count > 0u && input.text[i] != EOT		// EOT check makes it easier to write funs that do stuff like matching chars that are not something
			{
				uint::range(0u, count)
				{|_k|
					if input.text[i] == '\r'
					{
						line += 1;
					}
					else if input.text[i] == '\n' && (i == 0u || input.text[i-1u] != '\r')
					{
						line += 1;
					}
					i += 1u;
				}
			}
			else
			{
				let text = str::from_chars(vec::slice(input.text, input.index, i));
				result = log_ok("scan0", input, {new_state: {index: i, line: line with input}, value: text});
			}
		}
		result
	}
}

#[doc = "Like scan0 except that at least one character must be consumed."]
fn scan1(err_mesg: str, fun: fn@ ([char], uint) -> uint) -> parser<str>
{
	{|input: state|
		result::chain(scan0(fun)(input))
		{|pass|
			if pass.new_state.index > input.index
			{
				log_ok("scan1", input, pass)
			}
			else
			{
				log_err("scan1", input, {old_state: input, err_state: pass.new_state, mesg: err_mesg})
			}
		}
	}
}

#[doc = "Returns s if input matches s. Also see literalv."]
fn literal(s: str) -> parser<str>
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
			log_err(#fmt["text '%s'", s], input, {old_state: input, err_state: {index: j with input}, mesg: #fmt["Expected '%s'", s]})
		}
	}
}

#[doc = "Returns value if input matches s. Also see literal."]
fn literalv<T: copy>(s: str, value: T) -> parser<T>
{
	{|input: state|
		alt literal(s)(input)
		{
			result::ok(pass)
			{
				log_ok("literalv", input, {new_state: pass.new_state, value: value})
			}
			result::err(failure)
			{
				log_err(#fmt["literalv '%s'", s], input, failure)
			}
		}
	}
}

#[doc = "integer := [+-]? [0-9]+"]
fn integer() -> parser<int>
{
	let digits = match1(is_digit, "Expected digits").then({|s| return(option::get(int::from_str(s)))});
	let case1 = literal("+")._then(digits);
	let case2 = sequence2(literal("-"), digits, {|_o, v| result::ok(-v)});
	let case3 = digits;
	alternative([case1, case2, case3])
}

#[doc = "identifier := [a-zA-Z_] [a-zA-Z0-9_]*"]
fn identifier() -> parser<str>
{
	let prefix = match1(is_identifier_prefix, "Expected identifier");
	let suffix = match1(is_identifier_suffix, "Expected identifier").repeat0();
	prefix.then({|p| suffix.then({|s| return(p + str::connect(s, ""))})})
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
			log_err("eot", input, {old_state: input, err_state: input, mesg: "Expected EOT"})
		}
	}
}

#[doc = "Parses the text and fails if all the text was not consumed. Leading space is allowed.

This is typically used in conjunction with the parse function. Note that space has to have the
same type as parser which is backwards from how it is normally used. To get this to work you
can use a syntax like: `return(x).space0()` where x is of type T."]
fn everything<T: copy>(parser: parser<T>, space: parser<T>) -> parser<T>
{
	sequence3(space, parser, eot()) {|_a, b, _c| result::ok(b)}
}

#[doc = "These work the same as the functions of the same name, but tend
to make the code look a bit better."]
impl parser_methods<T: copy> for parser<T>
{
	fn space0<T: copy>() -> parser<T>
	{
		space0(self)
	}
	
	fn space1<T: copy>() -> parser<T>
	{
		space1(self)
	}

	fn everything<T: copy>(space: parser<T>) -> parser<T>
	{
		everything(self, space)
	}
}
