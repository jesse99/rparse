// Functions used by unit tests.
import io;
import io::writer_util;
import result = result::result;

import types::*;

#[cfg(test)]
fn check_ok<T: copy>(inText: str, parser: parser<T>, expected: T, line: int, seed: T) -> bool
{
	let text = chars_with_eot(inText);
	alt parser({file: "unit test", text: text, index: 0u, line: 1, value: seed})
	{
		result::ok(answer)
		{
			if answer.value != expected
			{
				io::stderr().write_line(#fmt["Expected %? but found %?", expected, answer.value]);
				ret false;
			}
			if answer.line != line
			{
				io::stderr().write_line(#fmt["Expected line %d but found line %d", line, answer.line]);
				ret false;
			}
			ret true;
		}
		result::err(error)
		{
			io::stderr().write_line(#fmt["Error: %s", error.mesg]);
			ret false;
		}
	}
}

#[cfg(test)]
fn check_err_status<T: copy>(x: status<T>, expected: str, line: int) -> bool
{
	alt x
	{
		result::ok(answer)
		{
			io::stderr().write_line(#fmt["Expected error '%s' but found %?", expected, answer.value]);
			ret false;
		}
		result::err(error)
		{
			if error.mesg != expected
			{
				io::stderr().write_line(#fmt["Expected error '%s' but found '%s'", expected, error.mesg]);
				ret false;
			}
			if error.output.line != line
			{
				io::stderr().write_line(#fmt["Expected error '%s' on line %d but line is %d", expected, line, error.output.line]);
				ret false;
			}
			ret true;
		}
	}
}

#[cfg(test)]
fn check_err_str<T: copy>(text: str, parser: str_parser<T>, expected: str, line: int) -> bool
{
	ret check_err_status(parser(text), expected, line);
}

#[cfg(test)]
fn check_err<T: copy>(inText: str, parser: parser<T>, expected: str, line: int, seed: T) -> bool
{
	let text = chars_with_eot(inText);
	let x= parser({file: "unit test", text: text, index: 0u, line: 1, value: seed});
	ret check_err_status(x, expected, line);
}
