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

import misc::*;
import types::*;

// ---- Debugging -------------------------------------------------------------
#[doc = "Used to log the results of a parse function (at info level)."]
fn log_ok<T: copy>(fun: str, input: state, result: succeeded<T>) -> status<T>
{
	// Note that we make multiple calls to munge_chars which is fairly slow, but
	// we only do that when actually logging: when info or debug logging is off
	// the munge_chars calls aren't evaluated.
	assert result.new_state.index >= input.index;			// can't go backwards on success (but no progress is fine, eg e*)
	if result.new_state.index > input.index
	{
		#info("%s", munge_chars(input.text));
		#info("%s^ %s parsed '%s'", repeat_char(' ', result.new_state.index), fun, str::slice(munge_chars(input.text), input.index, result.new_state.index));
	}
	else
	{
		#debug("%s", munge_chars(input.text));
		#debug("%s^ %s passed", repeat_char(' ', result.new_state.index), fun);
	}
	ret result::ok(result);
}

#[doc = "Used to log the results of a parse function (at debug level)."]
fn log_err<T: copy>(fun: str, input: state, result: failed<T>) -> status<T>
{
	assert result.old_state.index == input.index;			// on errors the next parser must begin at the start
	assert result.err_state.index >= input.index;			// errors can't be before the input

	#debug("%s", munge_chars(input.text));
	#debug("%s^ %s failed", repeat_char('-', input.index), fun);
	ret result::err(result);
}

// ---- Generators ------------------------------------------------------------
#[doc = "Used to log the results of a parse function (at debug level)."]
fn fails<T: copy>(mesg: str) -> parser<T>
{
	{|input: state|
		log_err("fails", input, {old_state: input, err_state: input, mesg: mesg})}
}

#[doc = "Returns a parser which always succeeds, but does not consume any input. 

Otherwise known as the monadic unit function."]
fn return<T: copy>(value: T) -> parser<T>
{
	{|input: state|
		log_ok("return", input, {new_state: input, value: value})}
}

#[doc = "Returns a parser which succeeds until EOT is reached."]
fn next() -> parser<char>
{
	{|input: state|
		let ch = input.text[input.index];
		if ch != EOT
		{
			log_ok("next", input, {new_state: {index: input.index + 1u with input}, value: ch})
		}
		else
		{
			log_err("next", input, {old_state: input, err_state: input, mesg: "EOT"})
		}
	}
}

// ---- Combinators -----------------------------------------------------------
impl basis_combinators<T: copy> for parser<T>
{
	#[doc = "If everything is successful then the function returned by eval is called
	with the result of calling self. If self fails eval is not called. Also see _then.
	
	Otherwise known as the monadic bind function. Often used to translate parsed
	values: `p().then({|v| return(blah::from_whatever(v))})`"]
	fn then<T: copy, U: copy>(eval: fn@ (T) -> parser<U>) -> parser<U>
	{
		{|input: state|
			result::chain(self(input))
			{|pass|
				result::chain_err(eval(pass.value)(pass.new_state))
				{|failure|
					log_err("then", input, {old_state: input with failure})
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
			{|failure1|
				result::chain_err(parser2(input))
				{|failure2|
					log_err("or", input, {mesg: failure1.mesg + " or " + failure2.mesg with failure2})
				}
			}
		}
	}
}
