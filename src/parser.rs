#[doc = "Generic parse functions."];

import io;
import io::writer_util;
import chain = result::chain;
import result = result::result;
import types::*;

// ---- Helper Functions ------------------------------------------------------
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
		ret result::ok(input);
	}
	else
	{
		ret chain(parsers[index](input)) {|out| sequence_r(out, parsers, index + 1u)};
	}
}

// ---- Parse Functions -------------------------------------------------------
// This (and some of the other functions) handle repetition themselves
// for efficiency. It also has a very short name because it is a very commonly
// used function.
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
	
	ret result::ok({index: i, line: line with input});
}

#[doc = "spaces := (' ' | '\t' | '\r' | '\n')+"]
fn spaces<T: copy>(input: state<T>) -> status<T>
{
	let result = s(input);
	let state = result::get(result);
	
	if state.index > input.index
	{
		ret result;
	}
	else
	{
		ret result::err({output: input, mesg: "expected whitespace"});
	}
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
		ret result::err({output: input, mesg: "expected an integer"});
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
			ret result::ok({value: value with answer});
		}
		result::err(error)
		{
			ret result::err(error);
		}
	}
}

#[doc = "sequence := e1 e2 e3…"]
fn sequence<T: copy>(input: state<T>, parsers: [parser<T>]) -> status<T>
{
	ret sequence_r(input, parsers, 0u);
}

#[doc = "just := e"]
fn just<T: copy>(file: str, text: str, parser: parser<T>, seed: T) -> status<T>
{
	ret parser({file: file, text: chars_with_eot(text), index: 0u, line: 1, value: seed});
}

#[doc = "everything := space e EOT"]
fn everything<T: copy>(file: str, text: str, space: parser<T>, parser: parser<T>, seed: T) -> status<T>
{
	let input = {file: file, text: chars_with_eot(text), index: 0u, line: 1, value: seed};
	ret sequence(input, [space, parser, eot(_)]);
}
