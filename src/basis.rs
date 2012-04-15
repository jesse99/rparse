#[doc = "Primitive parser generators and parser combinators.

Broadly speaking there are three kinds of functions exported by the library:
1. Parse functions take a state and return a status<T>. If the parse succeeded
then status<T> will be a succeeded<T> containing the value for the parse (a T) and
the index of the next character to be parsed. If the parse failed then status<T>
will be a failed<T> containing the input state and an error message.
2. Parser generators are functions that return brand-new parse functions, for
example fn return.
3) Parser combinators are functions that take one or more parse functions and
return a new parse function that composes the input arguments, for example fn or.
These are normally defined as implementation methods to make them nicer to use.

The idea is that you use parser generators and combinators to assemble a function
which can then be used to parse whatever you like. The functions within this module
are the primitive functions from which all the other functions are built.
"];

import types::*;

const EOT: char = '\u0003';

// ---- Generators ------------------------------------------------------------
#[doc = "Returns a parser which always fails. 

Otherwise known as the zero monadic value."]
fn fails<T: copy>(mesg: str) -> parser<T>
{
	{|input: state|
		result::err({new_state: input, max_index: input.index, mesg: mesg})}
}

#[doc = "Returns a parser which always succeeds, but does not consume any input. 

Otherwise known as the monadic unit function."]
fn return<T: copy>(value: T) -> parser<T>
{
	{|input: state|
		result::ok({new_state: input, value: value})}
}

#[doc = "Returns a parser which succeeds until EOT is reached."]
fn next() -> parser<char>
{
	{|input: state|
		let ch = input.text[input.index];
		if ch != EOT
		{
			result::ok({new_state: {index: input.index + 1u with input}, value: ch})
		}
		else
		{
			result::err({new_state: input, max_index: input.index, mesg: "EOT"})
		}
	}
}

// ---- Combinators -----------------------------------------------------------
impl parser_methods<T: copy> for parser<T>
{
	#[doc = "If everything is successful then the function returned by eval is called
	with the result of calling self. If self fails eval is not called. If you don't
	want to use the value from self _then can be used instead.
	
	Otherwise known as the monadic bind function."]
	fn then<T: copy, U: copy>(eval: fn@ (T) -> parser<U>) -> parser<U>
	{
		{|input: state|
			result::chain(self(input))
			{|output|
				result::chain_err(eval(output.value)(output.new_state))
				{|failure|
					result::err({new_state: input with failure})
				}
			}
		}
	}
	
	#[doc = "Returns a parser which first tries self, and if that fails, parser 2.
	
	Otherwise known as the monadic plus function."]
	fn or<T: copy>( parser2: parser<T>) -> parser<T>
	{
		{|input: state|
			result::chain_err(self(input))
			{|_failure|
				parser2(input)
			}
		}
	}
}
