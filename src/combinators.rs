#[doc = "Functions used to build parse function using parse functions."];

import misc::*;
import primitives::*;
import types::*;

impl std_combinators<T: copy> for parser<T>
{
	#[doc = "space0 := e [ \t\r\n]*"]
	fn space0<T: copy>() -> parser<T>
	{
		{|input: state|
			result::chain(self(input))
			{|pass|
				let mut i = pass.new_state.index;
				let mut line = pass.new_state.line;
				while true
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
				
				log_ok("space0", input, {new_state: {index: i, line: line with pass.new_state}, value: pass.value})
			}
		}
	}
	
	#[doc = "space1 := e [ \t\r\n]+"]
	fn space1<T: copy>() -> parser<T>
	{
		{|input: state|
			result::chain(self.space0()(input))
			{|pass|
				if option::is_some(str::find_char(" \t\r\n", input.text[pass.new_state.index - 1u]))	// little cheesy, but saves us from adding a helper fn
				{
					log_ok("space1", input, pass)
				}
				else
				{
					log_err("space1", input, {old_state: input, err_state: pass.new_state, mesg: "whitespace"})
				}
			}
		}
	}
}

