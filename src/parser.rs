#[doc = "Generic parse functions."];

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

// Kind of a crappy version
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
		ret result::err({output: answer, mesg: #fmt["expected EOT but found '%s'", trailer]});
	}
}

fn sequence_r<T: copy>(input: state<T>, parsers: [parser<T>], index: uint) -> status<T>
{
	if index == vec::len(parsers)
	{
		ret plog("sequence", input, result::ok(input));
	}
	else
	{
		ret plog("sequence", input, chain(parsers[index](input)) {|out| sequence_r(out, parsers, index + 1u)});
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
	ret plog("fails", input, result::err({output: input, mesg: "forced failure"}));
}

// This (and some of the other functions) handle repetition themselves
// for efficiency. It also has a very short name because it is a very commonly
// used function. TODO: could use a longer name if we decide to put it into
// state<T>.
#[doc = "s := (' ' | '\t' | '\r' | '\n')*"]
fn s<T: copy>(input: state<T>) -> status<T>
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

#[doc = "spaces := (' ' | '\t' | '\r' | '\n')+"]
fn spaces<T: copy>(input: state<T>) -> status<T>
{
	let result = s(input);
	let state = result::get(result);
	
	if state.index > input.index
	{
		ret plog("spaces", input, result);
	}
	else
	{
		ret plog("spaces", input, result::err({output: input, mesg: "expected whitespace"}));
	}
}

#[doc = "literal := <literal> space"]
fn literal<T: copy>(input: state<T>, literal: str, space: parser<T>) -> status<T>
{
	assert str::is_ascii(literal);		// so it's OK to cast literal to char, TODO: could relax this with an each_char that passed in both the char and its index
	
	let mut i = 0u;
	while i < str::len(literal)
	{
		if input.text[input.index + i] == literal[i] as char
		{
			i += 1u;
		}
		else
		{
			ret plog(#fmt["literal '%s'", literal], input, result::err({output: input, mesg: #fmt["expected '%s'", literal]}));
		}
	}
	
	ret plog("literal", input, space({index: input.index + i with input}));
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
		ret plog("integer", input, result::err({output: input, mesg: "expected an integer"}));
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

#[doc = "alternative := e1 | e2 | e3…"]
fn alternative<T: copy>(input: state<T>, parsers: [parser<T>]) -> status<T>
{
	let mut i = 0u;
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
				vec::push(messages, error.mesg);
			}
		}
		i += 1u;
	}
	
	ret plog("alternative", input, result::err({output: input, mesg: str::connect(messages, " or ")}));
}

#[doc = "sequence := e1 e2 e3…"]
fn sequence<T: copy>(input: state<T>, parsers: [parser<T>]) -> status<T>
{
	ret plog("sequence", input, sequence_r(input, parsers, 0u));
}

#[doc = "Parses with the aid of a pointer to a parser (useful for things like parenthesized expressions)."]
fn cyclic<T: copy>(input: state<T>, parser: @mut parser<T>) -> status<T>
{
	ret (*parser)(input);
}

#[doc = "just := e"]
fn just<T: copy>(file: str, text: str, parser: parser<T>, seed: T) -> status<T>
{
	#info["------------------------------------------"];
	#info["parsing '%s'", text];
	ret parser({file: file, text: chars_with_eot(text), index: 0u, line: 1, value: seed});
}

#[doc = "everything := space e EOT"]
fn everything<T: copy>(file: str, text: str, space: parser<T>, parser: parser<T>, seed: T) -> status<T>
{
	#info["------------------------------------------"];
	#info["parsing '%s'", text];
	let input = {file: file, text: chars_with_eot(text), index: 0u, line: 1, value: seed};
	ret sequence(input, [space, parser, eot(_)]);
}
