// Functions used by unit tests.
import io;
import io::writer_util;
import result = result::result;

import types::*;

#[cfg(test)]
fn check_ok<T>(inText: str, parser: parser<T>, expected: T, line: int) -> bool
{
	let text = chars_with_eot(inText);
	alt parser({file: "unit test", text: text, index: 0, line: 1})
	{
		result::ok(answer)
		{
			if answer.value != expected
			{
				io::stderr().write_line(#fmt["Expected %? but found line %?", expected, answer.value]);
				ret false;
			}
			if answer.output.line != line
			{
				io::stderr().write_line(#fmt["Expected line %d but found line %d", line, answer.output.line]);
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
fn check_err<T>(inText: str, parser: parser<T>, expected: str, line: int) -> bool
{
	let text = chars_with_eot(inText);
	alt parser({file: "unit test", text: text, index: 0, line: 1})
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
