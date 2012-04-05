#[doc = "Generic parse functions."];

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

// ---- Parse Functions -------------------------------------------------------
// This (and some of the other functions) handle repetition themselves
// for efficiency. It also has a very short name because it is a very commonly
// used function.
#[doc = "s := (' ' | '\t' | '\r' | '\n')*"]
fn s(input: state) -> status<()>
{
	let mut i = input.index;
	let mut line = input.line;
	while true
	{
		if input.text[i] == '\r' && input.text[i+1] == '\n'
		{
			line += 1;
			i += 1
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
		i += 1;
	}
	
	ret result::ok({output: {index: i, line: line with input}, value: ()});
}

#[doc = "spaces := (' ' | '\t' | '\r' | '\n')+"]
fn spaces(input: state) -> status<()>
{
	let result = s(input);
	let state = result::get(result);
	
	if state.output.index > input.index
	{
		ret result;
	}
	else
	{
		ret result::err({output: input, mesg: "expected whitespace"});
	}
}

#[cfg(unimplemented)]
#[doc = "literal := <literal> space"]
fn literal(input: state, literal: str, space: parser<()>) -> status<str>
{
	ret result::err({output: input, mesg: "not implemented"});
}

#[cfg(unimplemented)]
#[doc = "integer := [+-]? [0-9]+ space"]
fn integer(input: state, space: parser<()>) -> status<int>
{
	ret result::err({output: input, mesg: "not implemented"});
}

#[cfg(unimplemented)]
#[doc = "optional := e?"]
fn optional<T>(input: state, parser: parser<T>) -> status<T>
{
	ret result::err({output: input, mesg: "not implemented"});
}

#[cfg(unimplemented)]
#[doc = "repeat_zero := e*"]
fn repeat_zero<T>(input: state, parser: parser<T>) -> status<T>
{
	ret result::err({output: input, mesg: "not implemented"});
}

#[cfg(unimplemented)]
#[doc = "repeat_one := e+"]
fn repeat_one<T>(input: state, parser: parser<T>) -> status<T>
{
	ret result::err({output: input, mesg: "not implemented"});
}

#[cfg(unimplemented)]
#[doc = "alternative := e1 | e2 | …"]
fn alternative<T>(input: state, alternatives: [parser<T>]) -> status<T>
{
	ret result::err({output: input, mesg: "not implemented"});
}

#[cfg(unimplemented)]
#[doc = "list := elem (sep space elem)*"]
fn list<T>(input: state, elem: parser<T>, sep: str, space: parser<()>) -> status<[T]>
{
	ret result::err({output: input, mesg: "not implemented"});
}

#[cfg(unimplemented)]
#[doc = "terms := term ([ops] space term)*"]
fn terms<T>(input: state, term: parser<T>, ops: [str], space: parser<()>, evaluators: [fn (T, T) -> T]) -> status<T>
{
	ret result::err({output: input, mesg: "not implemented"});
}

#[cfg(unimplemented)]
#[doc = "everything := space e EOT"]
fn everything<T>(file: str, text: str, space: parser<()>, parser: parser<T>) -> status<[T]>
{
	let state = {file: "unit test", text: chars_with_eot(text), index: 0, line: 1};
	ret result::err({output: state, mesg: "not implemented"});
}
