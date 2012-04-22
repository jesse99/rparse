#[doc = "Various utility functions. 

Clients should not need to use most of these except for log_ok and log_err."];
import types::*;

const EOT: char = '\u0003';

#[doc = "Used to log the results of a parse function (at info level)."]
fn log_ok<T: copy>(fun: str, input: state, pass: succeeded<T>) -> status<T>
{
	// Note that we make multiple calls to munge_chars which is fairly slow, but
	// we only do that when actually logging: when info or debug logging is off
	// the munge_chars calls aren't evaluated.
	assert pass.new_state.index >= input.index;			// can't go backwards on success (but no progress is fine, eg e*)
	if pass.new_state.index > input.index
	{
		#info("%s", munge_chars(input.text));
		#info("%s^ %s parsed '%s'", repeat_char(' ', pass.new_state.index), fun, str::slice(munge_chars(input.text), input.index, pass.new_state.index));
	}
	else
	{
		#debug("%s", munge_chars(input.text));
		#debug("%s^ %s passed", repeat_char(' ', pass.new_state.index), fun);
	}
	ret result::ok(pass);
}

#[doc = "Used to log the results of a parse function (at debug level)."]
fn log_err<T: copy>(fun: str, input: state, failure: failed) -> status<T>
{
	assert failure.old_state.index == input.index;			// on errors the next parser must begin at the start
	assert failure.err_state.index >= input.index;			// errors can't be before the input
	
	#debug("%s", munge_chars(input.text));
	if failure.err_state.index > input.index 
	{
		#debug("%s^%s! %s failed", repeat_char('-', input.index), repeat_char(' ', failure.err_state.index - input.index), fun);
	}
	else
	{
		#debug("%s^ %s failed", repeat_char('-', input.index), fun);
	}
	ret result::err(failure);
}

#[doc = "Converts a string to an array of char and appends an EOT character."]
fn chars_with_eot(s: str) -> [char]
{
	let mut buf = [], i = 0u;
	let len = str::len(s);

	vec::reserve(buf, len + 1u);
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

#[doc = "Returns true if ch is in [a-zA-Z]."]
pure fn is_alpha(ch: char) -> bool
{
	ret (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z');
}

#[doc = "Returns true if ch is in [0-9]."]
pure fn is_digit(ch: char) -> bool
{
	ret ch >= '0' && ch <= '9';
}

#[doc = "Returns true if ch is_alpha or is_digit."]
pure fn is_alphanum(ch: char) -> bool
{
	ret is_alpha(ch) || is_digit(ch);
}

#[doc = "Returns true if ch is_alpha or '_'."]
pure fn is_identifier_prefix(ch: char) -> bool
{
	ret is_alpha(ch) || ch == '_';
}

#[doc = "Returns true if ch is_alpha, is_digit, or '_'."]
pure fn is_identifier_suffix(ch: char) -> bool
{
	ret is_identifier_prefix(ch) || is_digit(ch);
}

#[doc = "Returns true if ch is 7-bit ASCII and not a control character."]
pure fn is_print(ch: char) -> bool
{
	ret ch >= ' ' && ch <= '~';
}

#[doc = "Returns true if ch is ' ', '\t', '\r', or '\n'."]
pure fn is_whitespace(ch: char) -> bool
{
	ret ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n';
}

#[doc = "Returns a string with count ch characters."]
fn repeat_char(ch: char, count: uint) -> str
{
	let mut value = "";
	str::reserve(value, count);
	uint::range(0u, count) {|_i| str::push_char(value, ch);}
	ret value;
}

#[doc(hidden)]
fn get_col(text: [char], index: uint) -> uint
{
	let mut i = index;
	
	while i > 0u && text[i-1u] != '\n' && text[i-1u] != '\r'
	{
		i -= 1u;
	}
	
	ret index - i + 1u;
}

// Note that we don't want to escape control characters here because we need
// one code point to map to one printed character (so our log_ok arrows point to
// the right character).
#[doc = "Replaces non-is_print characters with '.'."]
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

