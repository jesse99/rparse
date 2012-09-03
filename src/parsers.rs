//! Functions and methods that return functions that are able to parse strings.
//!
//! These can be divided into parsers that return chars, strings, and generic Ts.
// TODO: probably should use individual modules for these, but the dependencies
// are painful (see https://github.com/mozilla/rust/issues/3352).
use misc::*;
use types::{Parser, State, Status};

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
			result::Ok({new_state: {index: i, ..input}, value: input.text[input.index]})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: i, ..input}, mesg: @~""})
		}
	}
}

/// Parse functions which return a character.
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
				result::Ok({new_state: {index: i, ..input}, value: input.text[input.index]})
			}
			else
			{
				result::Err({old_state: input, err_state: {index: i, ..input}, mesg: @fmt!("[%s]", s)})
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
				result::Ok({new_state: {index: i, ..input}, value: input.text[input.index]})
			}
			else
			{
				result::Err({old_state: input, err_state: {index: i, ..input}, mesg: @fmt!("[^%s]", s)})
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
		result::Ok({new_state: {index: i, ..input}, value: @text})
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
			result::Ok({new_state: {index: i, ..input}, value: @text})
		}
		else
		{
			result::Err({old_state: input, err_state: {index: i, ..input}, mesg: @~""})
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

/// Calls fun once and matches the number of characters returned by fun. 
/// 
/// This does increment line.  Note that this succeeds even if zero characters are matched.
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
			result::Ok({new_state: {index: i, line: line, ..input}, value: @text})
		}
		else
		{
			result::Ok({new_state: {index: i, line: line, ..input}, value: @~""})
		}
	}
}


/// If all the parsers are successful then the matched text is returned.
fn seq2_ret_str<T0: copy owned, T1: copy owned>(p0: Parser<T0>, p1: Parser<T1>) -> Parser<@~str>
{
	|input: State|
	{
		match p0.then(p1)(input)
		{
			result::Ok(pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok({new_state: pass.new_state, value: @text})
			}
			result::Err(failure) =>
			{
				result::Err({old_state: input, ..failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq3_ret_str<T0: copy owned, T1: copy owned, T2: copy owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>) -> Parser<@~str>
{
	|input: State|
	{
		match p0.then(p1). then(p2)(input)
		{
			result::Ok(pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok({new_state: pass.new_state, value: @text})
			}
			result::Err(failure) =>
			{
				result::Err({old_state: input, ..failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq4_ret_str<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>, p3: Parser<T3>) -> Parser<@~str>
{
	|input: State| {
		match p0.then(p1). then(p2).then(p3)(input)
		{
			result::Ok(pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok({new_state: pass.new_state, value: @text})
			}
			result::Err(failure) =>
			{
				result::Err({old_state: input, ..failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq5_ret_str<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, T4: copy owned>(p0: Parser<T0>, p1: Parser<T1>, p2: Parser<T2>, p3: Parser<T3>, p4: Parser<T4>) -> Parser<@~str>
{
	|input: State| {
		match p0.then(p1). then(p2).then(p3).then(p4)(input)
		{
			result::Ok(pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok({new_state: pass.new_state, value: @text})
			}
			result::Err(failure) =>
			{
				result::Err({old_state: input, ..failure})
			}
		}
	}
}

/// Parse functions which return a string.
trait StringParsers
{
	/// Returns the input that matches self. Also see liti and litv.
	fn lit() -> Parser<@~str>;
	
	/// Returns the input that matches lower-cased self. Also see lit and litv.
	fn liti() -> Parser<@~str>;
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
				result::Ok({new_state: {index: j, ..input}, value: @text})
			}
			else
			{
				result::Err({old_state: input, err_state: {index: j, ..input}, mesg: @fmt!("'%s'", s)})
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
				result::Ok({new_state: {index: j, ..input}, value: @text})
			}
			else
			{
				result::Err({old_state: input, err_state: {index: j, ..input}, mesg: @fmt!("'%s'", s)})
			}
		}
	}
}

// ---- generic parsers ---------------------------------------------------------------------------
/// Returns a parser which always fails.
fn fails<T: copy owned>(mesg: &str) -> Parser<T>
{
	let mesg = mesg.to_unique();
	|input: State| result::Err({old_state: input, err_state: input, mesg: @copy mesg})
}

/// Returns a parser which always succeeds, but does not consume any input.
fn ret<T: copy owned>(value: T) -> Parser<T>
{
	|input: State| result::Ok({new_state: input, value: value})
}

/// seq2 := e0 e1
fn seq2<T0: copy owned, T1: copy owned, R: copy owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, eval: fn@ (T0, T1) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
		match eval(a0, a1)
		{
			result::Ok(value) =>
			{
				ret(value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}
}

/// seq3 := e0 e1 e2
fn seq3<T0: copy owned, T1: copy owned, T2: copy owned, R: copy owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, eval: fn@ (T0, T1, T2) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
		match eval(a0, a1, a2)
		{
			result::Ok(value) =>
			{
				ret(value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}
}

/// seq4 := e0 e1 e2 e3
fn seq4<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, R: copy owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>, eval: fn@ (T0, T1, T2, T3) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
	do parser3.thene() |a3| {
		match eval(a0, a1, a2, a3)
		{
			result::Ok(value) =>
			{
				ret(value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}
}

/// seq5 := e0 e1 e2 e3 e4
fn seq5<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, T4: copy owned, R: copy owned>
	(parser0: Parser<T0>, parser1: Parser<T1>, parser2: Parser<T2>, parser3: Parser<T3>, parser4: Parser<T4>, eval: fn@ (T0, T1, T2, T3, T4) -> result::Result<R, @~str>) -> Parser<R>
{
	do parser0.thene() |a0| {
	do parser1.thene() |a1| {
	do parser2.thene() |a2| {
	do parser3.thene() |a3| {
	do parser4.thene() |a4| {
		match eval(a0, a1, a2, a3, a4)
		{
			result::Ok(value) =>
			{
				ret(value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}
}

/// seq6 := e0 e1 e2 e3 e4 e5
fn seq6<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, T4: copy owned, T5: copy owned, R: copy owned>
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
			result::Ok(value) =>
			{
				ret(value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}}
}

/// seq7 := e0 e1 e2 e3 e4 e5 e6
fn seq7<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, T4: copy owned, T5: copy owned, T6: copy owned, R: copy owned>
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
			result::Ok(value) =>
			{
				ret(value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}}}
}

/// seq8 := e0 e1 e2 e3 e4 e5 e6 e7
fn seq8<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, T4: copy owned, T5: copy owned, T6: copy owned, T7: copy owned, R: copy owned>
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
			result::Ok(value) =>
			{
				ret(value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}}}}
}

/// seq9 := e0 e1 e2 e3 e4 e5 e6 e7 e8
fn seq9<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, T4: copy owned, T5: copy owned, T6: copy owned, T7: copy owned, T8: copy owned, R: copy owned>
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
			result::Ok(value) =>
			{
				ret(value)
			}
			result::Err(mesg) =>
			{
				fails(*mesg)
			}
		}
	}}}}}}}}}
}

/// Parse functions which return a generic type.
trait GenericParsers
{
	/// Returns value if input matches s. Also see lit.
	fn litv<T: copy owned>(value: T) -> Parser<T>;
}

trait Combinators<T: copy owned>
{
	/// If parser1 is successful is successful then parser2 is called (and the value from parser1
	/// is ignored). If parser1 fails parser2 is not called.
	fn then<U: copy owned>(parser2: Parser<U>) -> Parser<U>;
	
	/// If parser is successful then the function returned by eval is called
	/// with parser's result. If parser fails eval is not called.
	/// 
	/// Often used to translate parsed values: `p().thene({|pvalue| return(2*pvalue)})`
	fn thene<U: copy owned>(eval: fn@ (T) -> Parser<U>) -> Parser<U>;
}

impl<T: copy owned> Parser<T> : Combinators<T>
{
	fn then<U: copy owned>(parser2: Parser<U>) -> Parser<U>
	{
		|input: State|
		{
			do result::chain(self(input))
			|pass|
			{
				do result::chain_err(parser2(pass.new_state))
					|failure| {result::Err({old_state: input, ..failure})}
			}
		}
	}
	
	fn thene<U: copy owned>(eval: fn@ (T) -> Parser<U>) -> Parser<U>
	{
		|input: State|
		{
			do result::chain(self(input))
			|pass|
			{
				do result::chain_err(eval(pass.value)(pass.new_state))
					|failure| {result::Err({old_state: input, ..failure})}
			}
		}
	}
}

impl &str : GenericParsers
{
	fn litv<T: copy owned>(value: T) -> Parser<T>
	{
		let s = self.to_unique();
		
		|input: State|
		{
			match s.lit()(input)
			{
				result::Ok(pass) =>
				{
					result::Ok({new_state: pass.new_state, value: value})
				}
				result::Err(failure) =>
				{
					result::Err(failure)
				}
			}
		}
	}
}
