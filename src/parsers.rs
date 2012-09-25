//! Functions and methods that return functions that are able to parse strings.
//!
//! These can be divided into parsers that return chars, strings, and generic Ts.
// TODO: probably should use individual modules for these, but the dependencies
// are painful (see https://github.com/mozilla/rust/issues/3352).
use misc::*;
use types::{Parser, State, Status, Succeeded, Failed};

/// Return type of parse function.
type ParseStatus<T: Copy Owned> = result::Result<T, ParseFailed>;

/// Returned by parse function on error. Line and col are both 1-based.
struct ParseFailed {file: @~str, line: uint, col: uint, mesg: @~str}

// ---- weird parsers -----------------------------------------------------------------------------
// Returns a parser which matches the end of the input.
// Clients should use everything instead of this.
#[doc(hidden)]
fn eot() -> Parser<()>
{
	|input: State|
	{
		if input.text[input.index] == EOT
		{
			result::Ok(Succeeded {new_state: State {index: input.index + 1u, ..input}, value: ()})
		}
		else
		{
			result::Err(Failed {old_state: input, err_state: input, mesg: @~"EOT"})
		}
	}
}

// ---- char parsers ------------------------------------------------------------------------------
/// Consumes a character which must satisfy the predicate.
/// Returns the matched character.
fn anycp(predicate: fn@ (char) -> bool) -> Parser<char>
{
	|input: State| {
		let mut i = input.index;
		if input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		if i > input.index
		{
			result::Ok(Succeeded {new_state: State {index: i, ..input}, value: input.text[input.index]})
		}
		else
		{
			result::Err(Failed {old_state: input, err_state: State {index: i, ..input}, mesg: @~""})
		}
	}
}

/// Parse methods which return a character.
trait CharParsers
{
	/// Attempts to match any character in self. If matched the char is returned.
	fn anyc() -> Parser<char>;
	
	/// Attempts to match no character in self. If matched the char is returned.
	fn noc() -> Parser<char>;
}

impl &str : CharParsers
{
	fn anyc() -> Parser<char>
	{
		// Note that we're handing this string off to a closure so we can't get rid of this copy
		// even if we make the impl on ~str.
		let s = self.to_unique();
		
		|input: State|
		{
			let mut i = input.index;
			if str::find_char(s, input.text[i]).is_some()
			{
				i += 1u;
			}
			
			if i > input.index
			{
				result::Ok(Succeeded {new_state: State {index: i, ..input}, value: input.text[input.index]})
			}
			else
			{
				result::Err(Failed {old_state: input, err_state: State {index: i, ..input}, mesg: @fmt!("[%s]", s)})
			}
		}
	}
	
	fn noc() -> Parser<char>
	{
		let s = self.to_unique();
		
		|input: State|
		{
			let mut i = input.index;
			if input.text[i] != EOT && str::find_char(s, input.text[i]).is_none()
			{
				i += 1u;
			}
			
			if i > input.index
			{
				result::Ok(Succeeded {new_state: State {index: i, ..input}, value: input.text[input.index]})
			}
			else
			{
				result::Err(Failed {old_state: input, err_state: State {index: i, ..input}, mesg: @fmt!("[^%s]", s)})
			}
		}
	}
}

// ---- string parsers ----------------------------------------------------------------------------
// It would be a lot more elegant if match0, match1, and co were removed
// and users relied on composition to build the sort of parsers that they
// want. However in practice this is not such a good idea:
// 1) Matching is a very common operation, but instead of something simple
// like:
//    match0(p)
// users would have to write something like:
//    match(p).r0().str()
// 2) Generating an array of characters and then converting them into a string
// is much slower than updating a mutable string.
// 3) Debugging a parser is simpler if users can use higher level building
// blocks (TODO: though maybe we can somehow ignore or collapse low
// level parsers when logging).

/// Consumes zero or more characters matching the predicate.
/// Returns the matched characters. 
/// 
/// Note that this does not increment line.
fn match0(predicate: fn@ (char) -> bool) -> Parser<@~str>
{
	|input: State|
	{
		let mut i = input.index;
		while input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		let text = str::from_chars(vec::slice(input.text, input.index, i));
		result::Ok(Succeeded {new_state: State {index: i, ..input}, value: @text})
	}
}

/// Consumes one or more characters matching the predicate.
/// Returns the matched characters. 
/// 
/// Note that this does not increment line.
fn match1(predicate: fn@ (char) -> bool) -> Parser<@~str>
{
	|input: State|
	{
		let mut i = input.index;
		while input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		if i > input.index
		{
			let text = str::from_chars(vec::slice(input.text, input.index, i));
			result::Ok(Succeeded {new_state: State {index: i, ..input}, value: @text})
		}
		else
		{
			result::Err(Failed {old_state: input, err_state: State {index: i, ..input}, mesg: @~""})
		}
	}
}

/// match1_0 := prefix+ suffix*
fn match1_0(prefix: fn@ (char) -> bool, suffix: fn@ (char) -> bool) -> Parser<@~str>
{
	let prefix = match1(prefix);
	let suffix = match0(suffix);
	prefix.thene(|p| suffix.thene(|s| ret(@(*p + *s))))
}

/// optional_str := e?
///
/// Returns an empty string on failure.
fn optional_str(parser: Parser<@~str>) -> Parser<@~str>
{
	|input: State|
	{
		match parser(input)
		{
			result::Ok(ref pass)		=> result::Ok(Succeeded {new_state: pass.new_state, value: pass.value}),
			result::Err(ref _failure)		=> result::Ok(Succeeded {new_state: input, value: @~""}),
		}
	}
}

/// Calls fun once and matches the number of characters returned by fun. 
/// 
/// This does increment line.  Note that this succeeds even if zero characters are matched.
///
/// # Fun's are typically written like this:
///
/// ~~~
/// fn to_new_line(chars: @[char], index: uint) -> uint
/// {
///     let mut i = index;
///     loop
///     {
///         // Chars will always have an EOT character. If we hit it then
///         // we failed to find a new-line character so match nothing. 
///         if chars[i] == EOT
///         {
///             return 0;
///         }
///         else if chars[i] == '\r' || chars[i] == '\n'
///         {
///             // Match all the characters up to, but not including, the first new line.
///             return i - index;
///         }
///         else
///         {
///             i += 1;
///         }
///     }
/// }
/// ~~~
fn scan(fun: fn@ (@[char], uint) -> uint) -> Parser<@~str>
{
	|input: State|
	{
		let mut i = input.index;
		let mut line = input.line;
		
		let count = fun(input.text, i);
		if count > 0u && input.text[i] != EOT		// EOT check makes it easier to write funs that do stuff like matching chars that are not something
		{
			for uint::range(0u, count)
			|_k| {
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
			let text = str::from_chars(vec::slice(input.text, input.index, i));
			result::Ok(Succeeded {new_state: State {index: i, line: line, ..input}, value: @text})
		}
		else
		{
			result::Ok(Succeeded {new_state: State {index: i, line: line, ..input}, value: @~""})
		}
	}
}


/// If all the parsers are successful then the matched text is returned.
fn seq2_ret_str<T0: Copy Owned, T1: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>) -> Parser<@~str>
{
	|input: State|
	{
		match p0.then(p1)(input)
		{
			result::Ok(ref pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok(Succeeded {new_state: pass.new_state, value: @text})
			}
			result::Err(ref failure) =>
			{
				result::Err(Failed {old_state: input, ..*failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq3_ret_str<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>) -> Parser<@~str>
{
	|input: State|
	{
		match p0.then(p1). then(p2)(input)
		{
			result::Ok(ref pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok(Succeeded {new_state: pass.new_state, value: @text})
			}
			result::Err(ref failure) =>
			{
				result::Err(Failed {old_state: input, ..*failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq4_ret_str<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>, p3: Parser<T3>) -> Parser<@~str>
{
	|input: State| {
		match p0.then(p1). then(p2).then(p3)(input)
		{
			result::Ok(ref pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok(Succeeded {new_state: pass.new_state, value: @text})
			}
			result::Err(ref failure) =>
			{
				result::Err(Failed {old_state: input, ..*failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq5_ret_str<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned, T4: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>, p3: Parser<T3>, p4: Parser<T4>) -> Parser<@~str>
{
	|input: State| {
		match p0.then(p1). then(p2).then(p3).then(p4)(input)
		{
			result::Ok(ref pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok(Succeeded {new_state: pass.new_state, value: @text})
			}
			result::Err(ref failure) =>
			{
				result::Err(Failed {old_state: input, ..*failure})
			}
		}
	}
}

/// Parse methods which return a string.
trait StringParsers
{
	/// Returns the input that matches self. Also see liti and litv.
	fn lit() -> Parser<@~str>;
	
	/// Returns the input that matches lower-cased self. Also see lit and litv.
	fn liti() -> Parser<@~str>;
	
	/// s0 := e [ \t\r\n]*
	fn s0() -> Parser<@~str>;
	
	/// s1 := e [ \t\r\n]+
	fn s1() -> Parser<@~str>;
}

impl &str : StringParsers
{
	fn lit() -> Parser<@~str>
	{
		let s = self.to_unique();
		
		|input: State|
		{
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
				result::Ok(Succeeded {new_state: State {index: j, ..input}, value: @text})
			}
			else
			{
				result::Err(Failed {old_state: input, err_state: State {index: j, ..input}, mesg: @fmt!("'%s'", s)})
			}
		}
	}
	
	fn liti() -> Parser<@~str>
	{
		let s = str::to_lower(self);
		
		|input: State|
		{
			let mut i = 0u;
			let mut j = input.index;
			while i < str::len(s)
			{
				let {ch, next} = str::char_range_at(s, i);
				if ch == lower_char(input.text[j])
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
				result::Ok(Succeeded {new_state: State {index: j, ..input}, value: @text})
			}
			else
			{
				result::Err(Failed {old_state: input, err_state: State {index: j, ..input}, mesg: @fmt!("'%s'", s)})
			}
		}
	}
	
	fn s0() -> Parser<@~str>
	{
		self.lit().s0()
	}
	
	fn s1() -> Parser<@~str>
	{
		self.lit().s1()
	}
}

// ---- generic parsers ---------------------------------------------------------------------------
/// Returns a parser which always fails.
fn fails<T: Copy Owned>(mesg: &str) -> Parser<T>
{
	let mesg = mesg.to_unique();
	|input: State| result::Err(Failed {old_state: input, err_state: input, mesg: @copy mesg})
}

/// Parses with the aid of a pointer to a parser (useful for things like parenthesized expressions).
///
/// # Usage is like this:
///
/// ~~~
/// // create a pointer that we can initialize later with the real expr parser
/// let expr_ptr = @mut ret(0i);
/// let expr_ref = forward_ref(expr_ptr);
/// 
/// // expr_ref can be used to parse expressions
/// 
/// // initialize the expr_ptr with the real parser
/// *expr_ptr = expr;
/// ~~~
fn forward_ref<T: Copy Owned>(parser: @mut Parser<T>) -> Parser<T>
{
	|input: State| (*parser)(input)
}

/// or_v := e0 | e1 | …
/// 
/// This is a version of or that is nicer to use when there are more than two alternatives.
fn or_v<T: Copy Owned>(parsers: @~[Parser<T>]) -> Parser<T>
{
	// A recursive algorithm would be a lot simpler, but it's not clear how that could
	// produce good error messages.
	assert vec::is_not_empty(*parsers);
	
	|input: State|
	{
		let mut result: Option<Status<T>> = None;
		let mut errors = ~[];
		let mut max_index = uint::max_value;
		let mut i = 0u;
		while i < vec::len(*parsers) && option::is_none(result)
		{
			match parsers[i](input)
			{
				result::Ok(ref pass) =>
				{
					result = option::Some(result::Ok(*pass));
				}
				result::Err(ref failure) =>
				{
					if failure.err_state.index > max_index || max_index == uint::max_value
					{
						errors = ~[failure.mesg];
						max_index = failure.err_state.index;
					}
					else if failure.err_state.index == max_index
					{
						vec::push(errors, failure.mesg);
					}
				}
			}
			i += 1u;
		}
		
		if option::is_some(result)
		{
			option::get(result)
		}
		else
		{
			let errs = do vec::filter(errors) |s| {str::is_not_empty(*s)};
			let mesg = at_connect(errs, ~" or ");
			result::Err(Failed {old_state: input, err_state: State {index: max_index, ..input}, mesg: @mesg})
		}
	}
}

/// Returns a parser which always succeeds, but does not consume any input.
#[allow(deprecated_mode)]		// TODO: probably need to use &T instead
fn ret<T: Copy Owned>(value: T) -> Parser<T>
{
	|input: State| result::Ok(Succeeded {new_state: input, value: value})
}

/// seq2 := e0 e1
fn seq2<T0: Copy Owned, T1: Copy Owned, R: Copy Owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, eval: fn@ (T0, T1) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
		match eval(a0, a1)
		{
			result::Ok(ref value) =>
			{
				ret(*value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}
}

/// seq3 := e0 e1 e2
fn seq3<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, R: Copy Owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, eval: fn@ (T0, T1, T2) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
		match eval(a0, a1, a2)
		{
			result::Ok(ref value) =>
			{
				ret(*value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}
}

/// seq4 := e0 e1 e2 e3
fn seq4<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned, R: Copy Owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>, eval: fn@ (T0, T1, T2, T3) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
	do parser3.thene() |a3| {
		match eval(a0, a1, a2, a3)
		{
			result::Ok(ref value) =>
			{
				ret(*value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}
}

/// seq5 := e0 e1 e2 e3 e4
fn seq5<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned, T4: Copy Owned, R: Copy Owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>, parser4: Parser<T4>, eval: fn@ (T0, T1, T2, T3, T4) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
	do parser3.thene() |a3| {
	do parser4.thene() |a4| {
		match eval(a0, a1, a2, a3, a4)
		{
			result::Ok(ref value) =>
			{
				ret(*value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}
}

/// seq6 := e0 e1 e2 e3 e4 e5
fn seq6<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned, T4: Copy Owned, T5: Copy Owned, R: Copy Owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>, parser4: Parser<T4>, parser5: Parser<T5>, eval: fn@ (T0, T1, T2, T3, T4, T5) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
	do parser3.thene() |a3| {
	do parser4.thene() |a4| {
	do parser5.thene() |a5| {
		match eval(a0, a1, a2, a3, a4, a5)
		{
			result::Ok(ref value) =>
			{
				ret(*value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}}
}

/// seq7 := e0 e1 e2 e3 e4 e5 e6
fn seq7<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned, T4: Copy Owned, T5: Copy Owned, T6: Copy Owned, R: Copy Owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>, parser4: Parser<T4>, parser5: Parser<T5>, parser6: Parser<T6>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
	do parser3.thene() |a3| {
	do parser4.thene() |a4| {
	do parser5.thene() |a5| {
	do parser6.thene() |a6| {
		match eval(a0, a1, a2, a3, a4, a5, a6)
		{
			result::Ok(ref value) =>
			{
				ret(*value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}}}
}

/// seq8 := e0 e1 e2 e3 e4 e5 e6 e7
fn seq8<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned, T4: Copy Owned, T5: Copy Owned, T6: Copy Owned, T7: Copy Owned, R: Copy Owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>, parser4: Parser<T4>, parser5: Parser<T5>, parser6: Parser<T6>, parser7: Parser<T7>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6, T7) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
	do parser3.thene() |a3| {
	do parser4.thene() |a4| {
	do parser5.thene() |a5| {
	do parser6.thene() |a6| {
	do parser7.thene() |a7| {
		match eval(a0, a1, a2, a3, a4, a5, a6, a7)
		{
			result::Ok(ref value) =>
			{
				ret(*value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}}}}
}

/// seq9 := e0 e1 e2 e3 e4 e5 e6 e7 e8
fn seq9<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned, T4: Copy Owned, T5: Copy Owned, T6: Copy Owned, T7: Copy Owned, T8: Copy Owned, R: Copy Owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>, parser4: Parser<T4>, parser5: Parser<T5>, parser6: Parser<T6>, parser7: Parser<T7>, parser8: Parser<T8>, eval: fn@ (T0, T1, T2, T3, T4, T5, T6, T7, T8) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
	do parser3.thene() |a3| {
	do parser4.thene() |a4| {
	do parser5.thene() |a5| {
	do parser6.thene() |a6| {
	do parser7.thene() |a7| {
	do parser8.thene() |a8| {
		match eval(a0, a1, a2, a3, a4, a5, a6, a7, a8)
		{
			result::Ok(ref value) =>
			{
				ret(*value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}}}}}
}

/// seq2_ret0 := e0 e1
fn seq2_ret0<T0: Copy Owned, T1: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>) -> Parser<T0>
{
	seq2(p0, p1, |a0, _a1| result::Ok(a0))
}

/// seq2_ret1 := e0 e1
fn seq2_ret1<T0: Copy Owned, T1: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>) -> Parser<T1>
{
	seq2(p0, p1, |_a0, a1| result::Ok(a1))
}

/// seq3_ret0 := e0 e1 e2
fn seq3_ret0<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>) -> Parser<T0>
{
	seq3(p0, p1, p2, |a0, _a1, _a2| result::Ok(a0))
}

/// seq3_ret1 := e0 e1 e2
fn seq3_ret1<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>) -> Parser<T1>
{
	seq3(p0, p1, p2, |_a0, a1, _a2| result::Ok(a1))
}

/// seq3_ret2 := e0 e1 e2
fn seq3_ret2<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>) -> Parser<T2>
{
	seq3(p0, p1, p2, |_a0, _a1, a2| result::Ok(a2))
}

/// seq4_ret0 := e0 e1 e2 e3
fn seq4_ret0<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>, p3: Parser<T3>) -> Parser<T0>
{
	seq4(p0, p1, p2, p3, |a0, _a1, _a2, _a3| result::Ok(a0))
}

/// seq4_ret1 := e0 e1 e2 e3
fn seq4_ret1<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>, p3: Parser<T3>) -> Parser<T1>
{
	seq4(p0, p1, p2, p3, |_a0, a1, _a2, _a3| result::Ok(a1))
}

/// seq4_ret2 := e0 e1 e2 e3
fn seq4_ret2<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>, p3: Parser<T3>) -> Parser<T2>
{
	seq4(p0, p1, p2, p3, |_a0, _a1, a2, _a3| result::Ok(a2))
}

/// seq4_ret3 := e0 e1 e2 e3
fn seq4_ret3<T0: Copy Owned, T1: Copy Owned, T2: Copy Owned, T3: Copy Owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>, p3: Parser<T3>) -> Parser<T3>
{
	seq4(p0, p1, p2, p3, |_a0, _a1, _a2, a3| result::Ok(a3))
}

// chain_suffix := (op e)*
#[doc(hidden)]
fn chain_suffix<T: Copy Owned, U: Copy Owned>(parser: Parser<T>, op: Parser<U>) -> Parser<@~[(U, T)]>
{
	let q = op.thene(
	|operator|
	{
		parser.thene(
		|value|
		{
			ret((operator, value))
		})
	});
	q.r0()
}

// When using tag it can be useful to use empty messages for interior parsers
// so we need to handle that case.
#[doc(hidden)]
fn or_mesg(mesg1: @~str, mesg2: @~str) -> @~str
{
	if str::is_not_empty(*mesg1) && str::is_not_empty(*mesg2)
	{
		@(*mesg1 + " or " + *mesg2)
	}
	else if str::is_not_empty(*mesg1)
	{
		mesg1
	}
	else if str::is_not_empty(*mesg2)
	{
		mesg2
	}
	else
	{
		@~""
	}
}

/// Parse methods which return a generic type.
trait GenericParsers
{
	/// Returns value if input matches s. Also see lit.
	fn litv<T: Copy Owned>(value: T) -> Parser<T>;
}

/// Parse methods used to compose parsers.
///
/// Note that these don't actually consume input (although the parsers they are invoked with normally will).
trait Combinators<T: Copy Owned>
{
	/// chainl1 := e (op e)*
	/// 
	/// Left associative binary operator. eval is called for each parsed op.
	fn chainl1<U: Copy Owned>(op: Parser<U>, eval: fn@ (T, U, T) -> T) -> Parser<T>;
	
	/// chainr1 := e (op e)*
	/// 
	/// Right associative binary operator. eval is called for each parsed op.
	fn chainr1<U: Copy Owned>(op: Parser<U>, eval: fn@ (T, U, T) -> T) -> Parser<T>;
	
	/// Like note except that the mesg is also used for error reporting.
	/// 
	/// If label is not empty then it is used if the previous parser completely failed to parse or if its error
	/// message was empty. Otherwise it suppresses errors from the parser (in favor of a later err function).
	/// Non-empty labels should look like \"expression\" or \"statement\".
	fn err(label: &str) -> Parser<T>;
	
	/// Parses the text and fails if all the text was not consumed. Leading space is allowed.
	/// 
	/// This is typically used in conjunction with the parse method. Note that space has to have the
	/// same type as parser which is backwards from how it is normally used.
	fn everything<U: Copy Owned>(space: Parser<U>) -> Parser<T>;
	
	/// list := e (sep e)*
	/// 
	/// Values for each parsed e are returned.
	fn list<U: Copy Owned>(sep: Parser<U>) -> Parser<@~[T]>;
	
	/// Logs the result of the previous parser.
	/// 
	/// If it was successful then the log is at INFO level. Otherwise it is at DEBUG level.
	/// Also see err method.
	fn note(mesg: &str) -> Parser<T>;
	
	/// optional := e?
	fn optional() -> Parser<Option<T>>;
	
	/// Returns a parser which first tries parser1, and if that fails, parser2.
	fn or(parser2: Parser<T>) -> Parser<T>;
	
	/// Uses parser to parse text. Also see everything method.
	fn parse(file: @~str, text: &str) -> ParseStatus<T>;
	
	/// Succeeds if parser matches input n to m times (inclusive).
	fn r(n: uint, m: uint) -> Parser<@~[T]>;
	
	/// r0 := e*
	/// 
	/// Values for each parsed e are returned.
	fn r0() -> Parser<@~[T]>;
	
	/// r1 := e+
	/// 
	/// Values for each parsed e are returned.
	fn r1() -> Parser<@~[T]>;
	
	/// s0 := e [ \t\r\n]*
	fn s0() -> Parser<T>;
	
	/// s1 := e [ \t\r\n]+
	fn s1() -> Parser<T>;
	
	/// If parser1 is successful is successful then parser2 is called (and the value from parser1
	/// is ignored). If parser1 fails parser2 is not called.
	fn then<U: Copy Owned>(parser2: Parser<U>) -> Parser<U>;
	
	/// If parser is successful then the function returned by eval is called
	/// with parser's result. If parser fails eval is not called.
	/// 
	/// Often used to translate parsed values: `p().thene({|pvalue| return(2*pvalue)})`
	fn thene<U: Copy Owned>(eval: fn@ (T) -> Parser<U>) -> Parser<U>;
}

impl<T: Copy Owned> Parser<T> : Combinators<T>
{
	fn chainl1<U: Copy Owned>(op: Parser<U>, eval: fn@ (T, U, T) -> T) -> Parser<T>
	{
		|input: State|
		{
			do result::chain(self(input))
			|pass|
			{
				match chain_suffix(self, op)(pass.new_state)
				{
					result::Ok(ref pass2) =>
					{
						let value = vec::foldl(pass.value, *pass2.value, |lhs: T, rhs: (U, T)| {eval(lhs, rhs.first(), rhs.second())});
						result::Ok(Succeeded {new_state: pass2.new_state, value: value})
					}
					result::Err(ref failure) =>
					{
						result::Err(Failed {old_state: input, ..*failure})
					}
				}
			}
		}
	}
	
	fn chainr1<U: Copy Owned>(op: Parser<U>, eval: fn@ (T, U, T) -> T) -> Parser<T>
	{
		|input: State|
		{
			do result::chain(self(input))
			|pass|
			{
				match chain_suffix(self, op)(pass.new_state)
				{
					result::Ok(ref pass2) =>
					{
						if vec::is_not_empty(*pass2.value)
						{
							// e1 and [(op1 e2), (op2 e3)]
							let e1 = pass.value;
							let terms = pass2.value;
							
							// e1 and [op1, op2] and [e2, e3]
							let (ops, parsers) = vec::unzip(copy *terms);
							
							// [op1, op2] and [e1, e2] and e3
							let e3 = vec::last(parsers);
							let parsers = ~[e1] + vec::slice(parsers, 0u, vec::len(parsers) - 1u);
							
							// [(e1 op1), (e2 op2)] and e3
							let terms = vec::zip(parsers, ops);
							
							let value = vec::foldr(terms, e3, {|&&lhs: (T, U), &&rhs| eval(lhs.first(), lhs.second(), rhs)});
							result::Ok(Succeeded {new_state: pass2.new_state, value: value})
						}
						else
						{
							result::Ok(Succeeded {new_state: pass2.new_state, value: pass.value})
						}
					}
					result::Err(ref failure) =>
					{
						result::Err(Failed {old_state: input ,.. *failure})
					}
				}
			}
		}
	}
	
	fn err(label: &str) -> Parser<T>
	{
		let label = label.to_unique();
		
		|input: State|
		{
			do result::chain_err((self.note(label))(input))
			|failure| 
			{
				if str::is_empty(label)
				{
					result::Err(Failed {mesg: @~"", ..failure})
				}
				else if failure.err_state.index == input.index || str::is_empty(*failure.mesg)
				{
					result::Err(Failed {mesg: @copy label, ..failure})
				}
				else
				{
					// If we managed to parse something then it is usually better to
					// use that error message. (If that's not what you want then use
					// empty strings there).
					result::Err(failure)
				}
			}
		}
	}
	
	fn everything<U: Copy Owned>(space: Parser<U>) -> Parser<T>
	{
		seq3_ret1(space, self, eot())
	}
	
	fn list<U: Copy Owned>(sep: Parser<U>) -> Parser<@~[T]>
	{
		let term = sep.then(self).r0();
		
		|input: State|
		{
			do result::chain(self(input))
			|pass|
			{
				match term(pass.new_state)
				{
					result::Ok(ref pass2) =>
					{
						result::Ok(Succeeded {value: @(~[pass.value] + *pass2.value), ..*pass2})
					}
					result::Err(ref failure) =>
					{
						result::Err(Failed {old_state: input, ..*failure})
					}
				}
			}
		}
	}
	
	fn note(mesg: &str) -> Parser<T>
	{
		let mesg = mesg.to_unique();
		
		|input: State|
		{
			match self(input)
			{
				result::Ok(ref pass) =>
				{
					// Note that we make multiple calls to munge_chars which is fairly slow, but
					// we only do that when actually logging: when info or debug logging is off
					// the munge_chars calls aren't evaluated.
					assert pass.new_state.index >= input.index;			// can't go backwards on success (but no progress is fine, eg e*)
					if pass.new_state.index > input.index
					{
						info!("%s", munge_chars(input.text));
						info!("%s^ %s parsed '%s'", repeat_char(' ', pass.new_state.index), mesg, str::slice(munge_chars(input.text), input.index, pass.new_state.index));
					}
					else
					{
						info!("%s", munge_chars(input.text));
						info!("%s^ %s passed", repeat_char(' ', pass.new_state.index), mesg);
					}
					result::Ok(*pass)
				}
				result::Err(ref failure) =>
				{
					assert failure.old_state.index == input.index;			// on errors the next parser must begin at the start
					assert failure.err_state.index >= input.index;			// errors can't be before the input
					
					debug!("%s", munge_chars(input.text));
					if failure.err_state.index > input.index 
					{
						debug!("%s^%s! %s failed", repeat_char('-', input.index), repeat_char(' ', failure.err_state.index - input.index), mesg);
					}
					else
					{
						debug!("%s^ %s failed", repeat_char('-', input.index), mesg);
					}
					result::Err(*failure)
				}
			}
		}
	}
	
	fn optional() -> Parser<Option<T>>
	{
		|input: State|
		{
			match self(input)
			{
				result::Ok(ref pass) =>
				{
					result::Ok(Succeeded {new_state: pass.new_state, value: option::Some(pass.value)})
				}
				result::Err(ref _failure) =>
				{
					result::Ok(Succeeded {new_state: input, value: option::None})
				}
			}
		}
	}
	
	fn or(parser2: Parser<T>) -> Parser<T>
	{
		|input: State|
		{
			do result::chain_err(self(input))
			|failure1|
			{
				do result::chain_err(parser2(input))
				|failure2|
				{
					if failure1.err_state.index > failure2.err_state.index
					{
						result::Err(failure1)
					}
					else if failure1.err_state.index < failure2.err_state.index
					{
						result::Err(failure2)
					}
					else
					{
						result::Err(Failed {mesg: or_mesg(failure1.mesg, failure2.mesg), ..failure2})
					}
				}
			}
		}
	}
	
	fn parse(file: @~str, text: &str) -> ParseStatus<T>
	{
		let chars = chars_with_eot(text);
		let input = State {file: file, text: chars, index: 0u, line: 1};
		match self(input)
		{
			result::Ok(ref pass) =>
			{
				result::Ok(pass.value)
			}
			result::Err(ref failure) =>
			{
				let col = get_col(chars, failure.err_state.index);
				result::Err(ParseFailed {file: failure.old_state.file, line: failure.err_state.line as uint, col: col, mesg: failure.mesg})
			}
		}
	}
	
	fn r(n: uint, m: uint) -> Parser<@~[T]>
	{
		|input: State|
		{
			let mut output = input;
			let mut values = ~[];
			loop
			{
				match self(output)
				{
					result::Ok(ref pass) =>
					{
						assert pass.new_state.index > output.index;	// must make progress to ensure loop termination
						output = pass.new_state;
						vec::push(values, pass.value);
					}
					result::Err(_) =>
					{
						break;
					}
				}
			}
			
			let count = vec::len(values);
			if n <= count && count <= m
			{
				result::Ok(Succeeded {new_state: output, value: @values})
			}
			else
			{
				result::Err(Failed {old_state: input, err_state: output, mesg: @~""})
			}
		}
	}
	
	fn r0() -> Parser<@~[T]>
	{
		self.r(0u, uint::max_value)
	}
	
	fn r1() -> Parser<@~[T]>
	{
		self.r(1u, uint::max_value)
	}
	
	fn s0() -> Parser<T>
	{
		// It would be simpler to write this with scan0, but scan0 is relatively inefficient
		// and s0 is typically called a lot.
		|input: State|
		{
			do result::chain(self(input))
			|pass|
			{
				let mut i = pass.new_state.index;
				let mut line = pass.new_state.line;
				loop
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
				
				result::Ok(Succeeded {new_state: State {index: i, line: line, ..pass.new_state}, value: pass.value})
			}
		}
	}
	
	fn s1() -> Parser<T>
	{
		|input: State|
		{
			do result::chain(self.s0()(input))
			|pass|
			{
				if option::is_some(str::find_char(" \t\r\n", input.text[pass.new_state.index - 1u]))	// little cheesy, but saves us from adding a helper fn
				{
					result::Ok(pass)
				}
				else
				{
					result::Err(Failed {old_state: input, err_state: pass.new_state, mesg: @~"whitespace"})
				}
			}
		}
	}
	
	fn then<U: Copy Owned>(parser2: Parser<U>) -> Parser<U>
	{
		|input: State|
		{
			do result::chain(self(input))
			|pass|
			{
				do result::chain_err(parser2(pass.new_state))
					|failure| {result::Err(Failed {old_state: input, ..failure})}
			}
		}
	}
	
	fn thene<U: Copy Owned>(eval: fn@ (T) -> Parser<U>) -> Parser<U>
	{
		|input: State|
		{
			do result::chain(self(input))
			|pass|
			{
				do result::chain_err(eval(pass.value)(pass.new_state))
					|failure| {result::Err(Failed {old_state: input, ..failure})}
			}
		}
	}
}

impl &str : GenericParsers
{
	fn litv<T: Copy Owned>(value: T) -> Parser<T>
	{
		let s = self.to_unique();
		
		|input: State|
		{
			match s.lit()(input)
			{
				result::Ok(ref pass) =>
				{
					result::Ok(Succeeded {new_state: pass.new_state, value: value})
				}
				result::Err(ref failure) =>
				{
					result::Err(*failure)
				}
			}
		}
	}
}
