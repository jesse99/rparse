#[doc = "Parser functions with char return types."];

#[doc = "Consumes a character which must satisfy the predicate.
Returns the matched character."]
fn match(predicate: fn@ (char) -> bool) -> parser<char>
{
	{|input: state|
		let mut i = input.index;
		if input.text[i] != EOT && predicate(input.text[i])
		{
			i += 1u;
		}
		
		if i > input.index
		{
			log_ok("match", input, {new_state: {index: i with input}, value: input.text[input.index]})
		}
		else
		{
			log_err("match", input, {old_state: input, err_state: {index: i with input}, mesg: ""})
		}
	}
}

#[doc = "Attempts to match any character in chars. If matched the char is returned."]
fn anyc(chars: str) -> parser<char>
{
	{|input: state|
		let mut i = input.index;
		if str::find_char(chars, input.text[i]).is_some()
		{
			i += 1u;
		}
		
		if i > input.index
		{
			log_ok("anyc", input, {new_state: {index: i with input}, value: input.text[input.index]})
		}
		else
		{
			log_err("anyc", input, {old_state: input, err_state: {index: i with input}, mesg: #fmt["Expected [%s]", chars]})
		}
	}
}

#[doc = "Attempts to match no character in chars. If matched the char is returned."]
fn noc(chars: str) -> parser<char>
{
	{|input: state|
		let mut i = input.index;
		if input.text[i] != EOT && str::find_char(chars, input.text[i]).is_none()
		{
			i += 1u;
		}
		
		if i > input.index
		{
			log_ok("noc", input, {new_state: {index: i with input}, value: input.text[input.index]})
		}
		else
		{
			log_err("noc", input, {old_state: input, err_state: {index: i with input}, mesg: #fmt["Expected [^%s]", chars]})
		}
	}
}

