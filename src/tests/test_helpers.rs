// Functions used by unit tests.
import io;
import io::writer_util;
import result = result::result;
import misc::*;
import types::*;

fn check_char_ok(inText: str, parser: parser<char>, expected: char) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1,});
	ret check_ok(result, expected);
}

fn check_char_failed(inText: str, parser: parser<char>, expected: str, line: int) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1});
	ret check_failed(result, expected, line);
}

fn check_int_ok(inText: str, parser: parser<int>, expected: int) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1});
	ret check_ok(result, expected);
}

fn check_int_failed(inText: str, parser: parser<int>, expected: str, line: int) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1});
	ret check_failed(result, expected, line);
}

fn check_str_ok(inText: str, parser: parser<str>, expected: str) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1,});
	ret check_ok(result, expected);
}

fn check_str_failed(inText: str, parser: parser<str>, expected: str, line: int) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1});
	ret check_failed(result, expected, line);
}

fn check_str_array_ok(inText: str, parser: parser<[str]>, expected: [str]) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1,});
	ret check_ok(result, expected);
}

fn check_str_array_failed(inText: str, parser: parser<[str]>, expected: str, line: int) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1});
	ret check_failed(result, expected, line);
}

// ---- Private Functions -----------------------------------------------------
fn check_ok<T: copy>(result: status<T>, expected: T) -> bool
{
	alt result
	{
		result::ok(pass)
		{
			if pass.value != expected
			{
				io::stderr().write_line(#fmt["Expected %? but found %?", expected, pass.value]);
				ret false;
			}
			ret true;
		}
		result::err(failure)
		{
			io::stderr().write_line(#fmt["Error: expected %s", failure.mesg]);
			ret false;
		}
	}
}

fn check_failed<T: copy>(result: status<T>, expected: str, line: int) -> bool
{
	alt result
	{
		result::ok(pass)
		{
			io::stderr().write_line(#fmt["Expected error '%s' but found %?", expected, pass.value]);
			ret false;
		}
		result::err(failure)
		{
			if failure.mesg != expected
			{
				io::stderr().write_line(#fmt["Expected error '%s' but found '%s'", expected, failure.mesg]);
				ret false;
			}
			if failure.err_state.line != line
			{
				io::stderr().write_line(#fmt["Expected error '%s' on line %d but line is %d", expected, line, failure.err_state.line]);
				ret false;
			}
			ret true;
		}
	}
}
