//! Various utility functions. 
//!
//! Clients should not need to use these.
const EOT: char = '\u0003';

pure fn at_connect(v: &[@~str], sep: &str) -> ~str
{
	let mut s = ~"", first = true;
	for vec::each(v) |ss|
	{
		if first {first = false;} else {unsafe {str::push_str(s, sep);}}
		unsafe {str::push_str(s, **ss)};
	}
	return s;
}

/// Converts a string to an array of char and appends an EOT character.
fn chars_with_eot(s: &str) -> @[char]
{
	do core::at_vec::build_sized(s.len() + 1)
	|push|
	{
		let mut i = 0u;
		let len = str::len(s);
		while i < len
		{
			let {ch, next} = str::char_range_at(s, i);
			assert next > i;
			push(ch);
			i = next;
		}
		push(EOT);
	}
}

/// Returns true if ch is in [a-zA-Z].
pure fn is_alpha(ch: char) -> bool
{
	return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z');
}

/// Returns true if ch is in [0-9].
pure fn is_digit(ch: char) -> bool
{
	return ch >= '0' && ch <= '9';
}

/// Returns true if ch is_alpha or is_digit.
pure fn is_alphanum(ch: char) -> bool
{
	return is_alpha(ch) || is_digit(ch);
}

/// Returns true if ch is 7-bit ASCII and not a control character.
pure fn is_print(ch: char) -> bool
{
	return ch >= ' ' && ch <= '~';
}

/// Returns true if ch is ' ', '\t', '\r', or '\n'.
pure fn is_whitespace(ch: char) -> bool
{
	return ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n';
}

/// Returns ch as lower case.
pure fn lower_char(ch: char) -> char
{
	if ch >= 'A' && ch <= 'Z'
	{
		('a' as uint + (ch as uint - 'A' as uint)) as char
	}
	else
	{
		ch
	}
}

/// Returns a string with count ch characters.
fn repeat_char(ch: char, count: uint) -> ~str
{
	let mut value = ~"";
	str::reserve(value, count);
	for uint::range(0u, count) |_i| { str::push_char(value, ch);}
	return value;
}

#[doc(hidden)]
fn get_col(text: @[char], index: uint) -> uint
{
	let mut i = index;
	
	while i > 0u && text[i-1u] != '\n' && text[i-1u] != '\r'
	{
		i -= 1u;
	}
	
	return index - i + 1u;
}

// Note that we don't want to escape control characters here because we need
// one code point to map to one printed character (so our log_ok arrows point to
// the right character).

/// Replaces non-is_print characters with '.'."
fn munge_chars(chars: @[char]) -> ~str
{
	// TODO: I'd like to use bullet here, but while io::println handles it correctly
	// the logging subsystem does not. See issue 2154.
	//let bullet = '\u2022';
	let bullet = '.';
	
	let mut value = ~"";
	str::reserve(value, vec::len(chars));
	for vec::each(chars) |ch| { str::push_char(value, if is_print(*ch) {*ch} else {bullet});}
	return value;
}
