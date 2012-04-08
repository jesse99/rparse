#[doc = "Generic parse functions."];		// TODO: expand on this a bit (eg mention top level functions)

import io;
import io::writer_util;
import chain = result::chain;
import success = result::success;
import result = result::result;
import types::*;

// ---- Helper Functions ------------------------------------------------------
// TODO: don't export these.

// This is designed to speed up parsing because the parsers don't have 
// to repeatedly verify that index is in range.
//
// Of course converting a string to a vector is not especially efficient, but
// it will be faster than handling utf-8 (unless we can guarantee that it
// is all 7-bit ASCII of course).
#[doc = "Like str::chars except that END OF TEXT (\u0003) is appended."]
fn chars_with_eot(s: str) -> [char]
{
    let mut buf = [], i = 0u;
    let len = str::len(s);
    while i < len
    {
        let {ch, next} = str::char_range_at(s, i);
        buf += [ch];
        i = next;
    }
    buf += ['\u0003'];
    ret buf;
}

// Note that, unlike the functions in the char module, these are 7-bit ASCII functions.
fn is_alpha(ch: char) -> bool
{
	ret (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z');
}

fn is_digit(ch: char) -> bool
{
	ret ch >= '0' && ch <= '9';
}

fn is_alphanum(ch: char) -> bool
{
	ret is_alpha(ch) || is_digit(ch);
}

fn is_print(ch: char) -> bool
{
	ret ch >= ' ' && ch <= '~';
}

fn repeat_char(ch: char, count: uint) -> str
{
	let mut value = "";
	str::reserve(value, count);
	uint::range(0u, count) {|_i| str::push_char(value, ch);}
	ret value;
}

// Note that we don't want to escape control characters here because we need
// one code point to map to one printed character (so our plog arrows point to
// the right character).
fn munge_chars(chars: [char]) -> str
{
	// TODO: I'd like to use bullet here, but while io::println handles it correctly
	// the logging subsystem does not. See issue 2154.
	//let bullet = '\u2022';
	let bullet = '*';
	
	let mut value = "";
	str::reserve(value, vec::len(chars));
	vec::iter(chars) {|ch| str::push_char(value, if is_print(ch) {ch} else {bullet});}
	ret value;
}

fn eot<T: copy>(answer: state<T>) -> status<T>
{
	if answer.text[answer.index] == '\u0003'
	{
		ret result::ok(answer);
	}
	else
	{
		let last = uint::min(answer.index + 16u, vec::len(answer.text) - 1u);
		let trailer = str::from_chars(vec::slice(answer.text, answer.index, last));
		ret result::err({output: answer, maxIndex: answer.index, mesg: #fmt["expected EOT but found '%s'", trailer]});
	}
}

// ---- Parse Functions -------------------------------------------------------
#[doc = "Used to log the results of a parse function (at both info and debug levels).

Typical usage is to call this function with whatever the parse function wants to return:

   ret plog(\"my_parser\", input, output);"]
fn plog<T: copy>(fun: str, input: state<T>, output: status<T>) -> status<T>
{
	alt output
	{
		result::ok(answer)
		// Note that we make multiple calls to munge_chars which is fairly slow, but
		// we only do that when actually logging: when info or debug logging is off
		// the munge_chars calls aren't evaluated.
		{
			if answer.index > input.index
			{
				#info("%s", munge_chars(input.text));
				#info("%s^ %s parsed '%s'", repeat_char(' ', answer.index), fun, str::slice(munge_chars(input.text), input.index, answer.index));
			}
			else
			{
				#debug("%s", munge_chars(input.text));
				#debug("%s^ %s passed", repeat_char(' ', answer.index), fun);
			}
		}
		result::err(error)
		{
			#debug("%s", munge_chars(input.text));
			#debug("%s^ %s failed", repeat_char('-', input.index), fun);
		}
	}
	ret output;
}

#[doc = "A parser that always fails."]
fn fails(input: state<int>) -> status<int>
{
	ret plog("fails", input, result::err({output: input, maxIndex: input.index, mesg: "forced failure"}));
}

// This (and some of the other functions) handle repetition themselves
// for efficiency. It also has a very short name because it is a very commonly
// used function.
#[doc = "space_zero_or_more := (' ' | '\t' | '\r' | '\n')*"]
fn space_zero_or_more<T: copy>(input: state<T>) -> status<T>
{
	let mut i = input.index;
	let mut line = input.line;
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
	
	ret plog("s", input, result::ok({index: i, line: line with input}));
}

#[doc = "space_zero_or_more := (' ' | '\t' | '\r' | '\n')+"]
fn space_one_or_more<T: copy>(input: state<T>) -> status<T>
{
	let result = space_zero_or_more(input);
	let state = result::get(result);
	
	if state.index > input.index
	{
		ret plog("spaces", input, result);
	}
	else
	{
		ret plog("spaces", input, result::err({output: input, maxIndex: input.index, mesg: "expected whitespace"}));
	}
}

#[doc = "literal := <literal> space"]
fn literal<T: copy>(input: state<T>, literal: str, space: parser<T>) -> status<T>
{
	let mut i = 0u;
	let mut j = input.index;
	while i < str::len(literal)
	{
		let {ch, next} = str::char_range_at(literal, i);
		if ch == input.text[j]
		{
			i = next;
			j += 1u;
		}
		else
		{
			ret plog(#fmt["literal '%s'", literal], input, result::err({output: input, maxIndex: j, mesg: #fmt["expected '%s'", literal]}));
		}
	}
	
	ret plog("literal", input, space({index: j with input}));
}

#[doc = "integer := [+-]? [0-9]+ space"]
fn integer(input: state<int>, space: parser<int>) -> status<int>
{
	let mut start = input.index;
	if input.text[start] == '+' || input.text[start] == '-'
	{
		start += 1u;
	}
	
	let mut i = start;
	while is_digit(input.text[i])
	{
		i += 1u;
	}
	
	if i == start
	{
		ret plog("integer", input, result::err({output: input, maxIndex: start, mesg: "expected an integer"}));
	}
	
	alt space({index: i with input})		// TODO: not sure if we can simplify this with chain (type inference has problems figuring out the type of the closure)
	{
		result::ok(answer)
		{
			let text = str::from_chars(vec::slice(input.text, start, i));
			let mut value = option::get(int::from_str(text));
			if input.text[input.index] == '-'
			{
				value = -value;
			}
			ret plog("integer", input, result::ok({value: value with answer}));
		}
		result::err(error)
		{
			ret plog("integer", input, result::err(error));
		}
	}
}

#[doc = "optional := e?"]
fn optional<T: copy>(input: state<T>, parser: parser<T>) -> status<T>
{
	let result = parser(input);
	alt result
	{
		result::ok(answer)
		{
			ret plog("optional", input, result);
		}
		result::err(error)
		{
			ret plog("optional", input, result::ok(input));
		}
	}
}

#[doc = "alternative := e1 | e2 | e3…"]
fn alternative<T: copy>(input: state<T>, parsers: [parser<T>]) -> status<T>
{
	let mut i = 0u;
	let mut maxIndex = input.index;
	let mut errMesg = "";
	let mut messages: [str] = [];
	
	while i < vec::len(parsers)
	{
		let result = parsers[i](input);
		alt result
		{
			result::ok(answer)
			{
				ret plog("alternative", input, result);
			}
			result::err(error)
			{
				if error.maxIndex > maxIndex
				{
					maxIndex = error.maxIndex;
					errMesg = error.mesg;
				}
				else
				{
					vec::push(messages, error.mesg);
				}
			}
		}
		i += 1u;
	}
	
	// If the alternatives were able to process anything then we'll use the error message of the one that processed the most.
	// Otherwise none of them were able to process anything so we'll print what each expected.
	if str::is_empty(errMesg)
	{
		ret plog("alternative", input, result::err({output: input, maxIndex: maxIndex, mesg: str::connect(messages, " or ")}));
	}
	else
	{
		ret plog("alternative", input, result::err({output: input, maxIndex: maxIndex, mesg: errMesg}));
	}
}

#[doc = "sequence := e1 e2 e3…

Eval will be called with [input value, e1 value, …]."]
fn sequence<T: copy>(input: state<T>, parsers: [parser<T>], eval: fn@ ([T]) -> T) -> status<T>
{
	let mut results: [T] = [];
	vec::reserve(results, vec::len(parsers) + 1u);
	vec::push(results, input.value);
	
	let mut i = 0u;
	let mut out = input;
	while i < vec::len(parsers)
	{
		alt parsers[i](out)
		{
			result::ok(answer)
			{
				out = answer;
				vec::push(results, answer.value);
			}
			result::err(error)
			{
				ret plog("sequence", input, result::err(error));
			}
		}
		i += 1u;
	}
	
	ret plog("sequence", input, result::ok({value: eval(results) with out}));
}

#[doc = "Parses with the aid of a pointer to a parser (useful for things like parenthesized expressions)."]
fn cyclic<T: copy>(input: state<T>, parser: @mut parser<T>) -> status<T>
{
	ret (*parser)(input);
}

#[doc = "Parses the text and does not fail if all the text was not consumed.."]
fn just<T: copy>(file: str, parser: parser<T>, seed: T, text: str) -> status<T>
{
	#info["------------------------------------------"];
	#info["parsing '%s'", text];
	ret parser({file: file, text: chars_with_eot(text), index: 0u, line: 1, value: seed});
}

#[doc = "Parses the text and fails if all the text was not consumed. Leading space is allowed."]
fn everything<T: copy>(file: str, parser: parser<T>, space: parser<T>, seed: T, text: str) -> status<T>
{
	#info["------------------------------------------"];
	#info["parsing '%s'", text];
	let input = {file: file, text: chars_with_eot(text), index: 0u, line: 1, value: seed};
	ret sequence(input, [space, parser, eot(_)]) {|results| results[2]};
}
