#[doc = "Parser functions with generic return types."];

#[doc = "Returns value if input matches s. Also see lit."]
fn litv<T: copy>(s: str, value: T) -> parser<T>
{
	{|input: state|
		alt lit(s)(input)
		{
			result::ok(pass)
			{
				log_ok("litv", input, {new_state: pass.new_state, value: value})
			}
			result::err(failure)
			{
				log_err(#fmt["litv '%s'", s], input, failure)
			}
		}
	}
}

#[doc = "Returns a parser which always fails."]
fn fails<T: copy>(mesg: str) -> parser<T>
{
	{|input: state|
		log_err("fails", input, {old_state: input, err_state: input, mesg: mesg})}
}

#[doc = "Returns a parser which always succeeds, but does not consume any input."]
fn return<T: copy>(value: T) -> parser<T>
{
	{|input: state|
		log_ok(#fmt["return %?", value], input, {new_state: input, value: value})}
}

