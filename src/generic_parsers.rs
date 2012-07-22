#[doc = "Parser functions with generic return types."];

#[doc = "Returns value if input matches s. Also see lit."]
fn litv<T: copy owned>(s: &str, value: T) -> parser<T>
{
	let s = unslice(s);
	
	{|input: state|
		alt lit(s)(input)
		{
			result::ok(pass)
			{
				result::ok({new_state: pass.new_state, value: value})
			}
			result::err(failure)
			{
				result::err(failure)
			}
		}
	}
}

#[doc = "Returns a parser which always fails."]
fn fails<T: copy owned>(mesg: &str) -> parser<T>
{
	let mesg = unslice(mesg);
	
	{|input: state|
		result::err({old_state: input, err_state: input, mesg: unslice(mesg)})
	}
}

#[doc = "Returns a parser which always succeeds, but does not consume any input."]
fn return<T: copy owned>(value: T) -> parser<T>
{
	{|input: state|
		result::ok({new_state: input, value: value})
	}
}

