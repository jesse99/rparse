#[doc = "Various utlity functions."];

//import io;
//import io::writer_util;
//import result::*;
//import types::*;

const EOT: char = '\u0003';

// TODO: don't export these.

fn chars_with_eot(s: str) -> [char]
{
    let mut buf = [], i = 0u;
    let len = str::len(s);
    while i < len
    {
        let {ch, next} = str::char_range_at(s, i);
        assert next > i;
        buf += [ch];
        i = next;
    }
    buf += [EOT];
    ret buf;
}

// Note that, unlike the functions in the char module, these are 7-bit ASCII functions.
pure fn is_alpha(ch: char) -> bool
{
	ret (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z');
}

pure fn is_digit(ch: char) -> bool
{
	ret ch >= '0' && ch <= '9';
}

pure fn is_alphanum(ch: char) -> bool
{
	ret is_alpha(ch) || is_digit(ch);
}

pure fn is_print(ch: char) -> bool
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
// one code point to map to one printed character (so our log_ok arrows point to
// the right character).
fn munge_chars(chars: [char]) -> str
{
	// TODO: I'd like to use bullet here, but while io::println handles it correctly
	// the logging subsystem does not. See issue 2154.
	//let bullet = '\u2022';
	let bullet = '.';
	
	let mut value = "";
	str::reserve(value, vec::len(chars));
	vec::iter(chars) {|ch| str::push_char(value, if is_print(ch) {ch} else {bullet});}
	ret value;
}

