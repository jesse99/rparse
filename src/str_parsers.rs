//! Parser functions with str return types.

/// Returns s if input matches s ignoring case. Also see lit and litv.
fn liti(in_s: &str) -> parser<~str>
{
	let s = str::to_lower(in_s);
	
	|input: state| {
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
			result::ok({new_state: {index: j with input}, value: text})
		}
		else
		{
			result::err({old_state: input, err_state: {index: j with input}, mesg: #fmt["'%s'", s]})
		}
	}
}

/// Returns s if input matches s. Also see liti and litv.
fn lit(s: &str) -> parser<~str>
{
	let s = unslice(s);
	
	|input: state|
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
			result::ok({new_state: {index: j with input}, value: text})
		}
		else
		{
			result::err({old_state: input, err_state: {index: j with input}, mesg: #fmt["'%s'", unslice(s)]})
		}
	}
}

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
fn match0(predicate: fn@ (char) -> bool) -> parser<~str>
{
	|input: state| {
		let mut i = input.index;
		while input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		let text = str::from_chars(vec::slice(input.text, input.index, i));
		result::ok({new_state: {index: i with input}, value: text})
	}
}

/// Consumes one or more characters matching the predicate.
/// Returns the matched characters. 
/// 
/// Note that this does not increment line.
fn match1(predicate: fn@ (char) -> bool) -> parser<~str>
{
	|input: state| {
		let mut i = input.index;
		while input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		if i > input.index
		{
			let text = str::from_chars(vec::slice(input.text, input.index, i));
			result::ok({new_state: {index: i with input}, value: text})
		}
		else
		{
			result::err({old_state: input, err_state: {index: i with input}, mesg: ~""})
		}
	}
}

/// match1_0 := prefix+ suffix*
fn match1_0(prefix: fn@ (char) -> bool, suffix: fn@ (char) -> bool) -> parser<~str>
{
	let prefix = match1(prefix);
	let suffix = match0(suffix);
	prefix.thene(|p| suffix.thene(|s| return(p + s) ) )
}

/// optional := e?
fn optional_str(parser: parser<~str>) -> parser<~str>
{
	|input: state| {
		alt parser(input)
		{
			result::ok(pass)
			{
				result::ok({new_state: pass.new_state, value: pass.value})
			}
			result::err(_failure)
			{
				result::ok({new_state: input, value: ~""})
			}
		}
	}
}

/// Calls fun once and matches the number of characters returned by fun. 
/// 
/// This does increment line.
fn scan(fun: fn@ (@[char], uint) -> uint) -> parser<~str>
{
	|input: state| {
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
			result::ok({new_state: {index: i, line: line with input}, value: text})
		}
		else
		{
			result::ok({new_state: {index: i, line: line with input}, value: ~""})
		}
	}
}

/// Calls fun with an index into the characters to be parsed until it returns zero characters.
/// Returns the matched characters. 
/// 
/// This does increment line.
fn scan0(fun: fn@ (@[char], uint) -> uint) -> parser<~str>
{
	|input: state| {
		let mut i = input.index;
		let mut line = input.line;
		let mut result = result::err({old_state: input, err_state: input, mesg: ~"dummy"});
		while result::is_err(result)
		{
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
			}
			else
			{
				let text = str::from_chars(vec::slice(input.text, input.index, i));
				result = result::ok({new_state: {index: i, line: line with input}, value: text});
			}
		}
		result
	}
}

/// Like scan0 except that at least one character must be consumed.
fn scan1(fun: fn@ (@[char], uint) -> uint) -> parser<~str>
{
	|input: state| {
		do result::chain(scan0(fun)(input))
		|pass| {
			if pass.new_state.index > input.index
			{
				result::ok(pass)
			}
			else
			{
				result::err({old_state: input, err_state: pass.new_state, mesg: ~""})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq2_ret_str<T0: copy owned, T1: copy owned>(p0: parser<T0>, p1: parser<T1>) -> parser<~str>
{
	|input: state| {
		alt p0.then(p1)(input)
		{
			result::ok(pass)
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::ok({new_state: pass.new_state, value: text})
			}
			result::err(failure)
			{
				result::err({old_state: input with failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq3_ret_str<T0: copy owned, T1: copy owned, T2: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>) -> parser<~str>
{
	|input: state| {
		alt p0.then(p1). then(p2)(input)
		{
			result::ok(pass)
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::ok({new_state: pass.new_state, value: text})
			}
			result::err(failure)
			{
				result::err({old_state: input with failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq4_ret_str<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<~str>
{
	|input: state| {
		alt p0.then(p1). then(p2).then(p3)(input)
		{
			result::ok(pass)
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::ok({new_state: pass.new_state, value: text})
			}
			result::err(failure)
			{
				result::err({old_state: input with failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq5_ret_str<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, T4: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>, p4: parser<T4>) -> parser<~str>
{
	|input: state| {
		alt p0.then(p1). then(p2).then(p3).then(p4)(input)
		{
			result::ok(pass)
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::ok({new_state: pass.new_state, value: text})
			}
			result::err(failure)
			{
				result::err({old_state: input with failure})
			}
		}
	}
}
