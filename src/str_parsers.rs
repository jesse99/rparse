//! Parser functions with str return types.
use misc::*;
use types::*;

/// optional := e?
fn optional_str(parser: parser<~str>) -> parser<~str>
{
	|input: state| {
		match parser(input)
		{
			result::Ok(pass) =>
			{
				result::Ok({new_state: pass.new_state, value: pass.value})
			}
			result::Err(_failure) =>
			{
				result::Ok({new_state: input, value: ~""})
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
			result::Ok({new_state: {index: i, line: line ,.. input}, value: text})
		}
		else
		{
			result::Ok({new_state: {index: i, line: line ,.. input}, value: ~""})
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
		let mut result = result::Err({old_state: input, err_state: input, mesg: ~"dummy"});
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
				result = result::Ok({new_state: {index: i, line: line ,.. input}, value: text});
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
				result::Ok(pass)
			}
			else
			{
				result::Err({old_state: input, err_state: pass.new_state, mesg: ~""})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq2_ret_str<T0: copy owned, T1: copy owned>(p0: parser<T0>, p1: parser<T1>) -> parser<~str>
{
	|input: state| {
		match p0.then(p1)(input)
		{
			result::Ok(pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok({new_state: pass.new_state, value: text})
			}
			result::Err(failure) =>
			{
				result::Err({old_state: input ,.. failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq3_ret_str<T0: copy owned, T1: copy owned, T2: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>) -> parser<~str>
{
	|input: state| {
		match p0.then(p1). then(p2)(input)
		{
			result::Ok(pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok({new_state: pass.new_state, value: text})
			}
			result::Err(failure) =>
			{
				result::Err({old_state: input ,.. failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq4_ret_str<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>) -> parser<~str>
{
	|input: state| {
		match p0.then(p1). then(p2).then(p3)(input)
		{
			result::Ok(pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok({new_state: pass.new_state, value: text})
			}
			result::Err(failure) =>
			{
				result::Err({old_state: input ,.. failure})
			}
		}
	}
}

/// If all the parsers are successful then the matched text is returned.
fn seq5_ret_str<T0: copy owned, T1: copy owned, T2: copy owned, T3: copy owned, T4: copy owned>(p0: parser<T0>, p1: parser<T1>, p2: parser<T2>, p3: parser<T3>, p4: parser<T4>) -> parser<~str>
{
	|input: state| {
		match p0.then(p1). then(p2).then(p3).then(p4)(input)
		{
			result::Ok(pass) =>
			{
				let text = str::from_chars(vec::slice(input.text, input.index, pass.new_state.index));
				result::Ok({new_state: pass.new_state, value: text})
			}
			result::Err(failure) =>
			{
				result::Err({old_state: input ,.. failure})
			}
		}
	}
}
