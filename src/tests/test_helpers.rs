// Functions used by unit tests.
//use io;
use io::WriterUtil;
use misc::*;
use types::*;
use Result = result::Result;

fn check_char_ok(inText: &str, parser: parser<char>, expected: char) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1,});
	return check_ok(result, expected);
}

fn check_char_failed(inText: &str, parser: parser<char>, expected: &str, line: int) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1});
	return check_failed(result, expected, line);
}

fn check_int_ok(inText: &str, parser: parser<int>, expected: int) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1});
	return check_ok(result, expected);
}

fn check_int_failed(inText: &str, parser: parser<int>, expected: &str, line: int) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1});
	return check_failed(result, expected, line);
}

fn check_float_ok(inText: &str, parser: parser<f64>, expected: f64) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1});
	return check_ok(result, expected);
}

fn check_float_failed(inText: &str, parser: parser<f64>, expected: &str, line: int) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1});
	return check_failed(result, expected, line);
}

fn check_str_ok(inText: &str, parser: parser<@~str>, expected: &str) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1,});
	return check_ok_strs(result, expected);
}

fn check_str_failed(inText: &str, parser: parser<@~str>, expected: &str, line: int) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1});
	return check_failed(result, expected, line);
}

fn check_str_array_ok(inText: &str, parser: parser<@~[~str]>, expected: &[~str]) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1,});
	return check_ok_str_arrays(result, expected);
}

fn check_str_array_failed(inText: &str, parser: parser<@~[~str]>, expected: &str, line: int) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser({file: @~"unit test", text: text, index: 0u, line: 1});
	return check_failed(result, expected, line);
}

// ---- Private Functions -----------------------------------------------------
fn check_ok<T: copy owned>(result: status<T>, expected: T) -> bool
{
	match result
	{
		result::Ok(pass) =>
		{
			if pass.value != expected
			{
				io::stderr().write_line(fmt!("Expected %? but found %?", expected, pass.value));
				return false;
			}
			return true;
		}
		result::Err(failure) =>
		{
			io::stderr().write_line(fmt!("Error: expected %? but found error %s", expected, *failure.mesg));
			return false;
		}
	}
}

fn check_ok_strs(result: status<@~str>, expected: &str) -> bool
{
	match result
	{
		result::Ok(pass) =>
		{
			if *pass.value != expected.to_unique()
			{
				io::stderr().write_line(fmt!("Expected %? but found %?", expected, pass.value));
				return false;
			}
			return true;
		}
		result::Err(failure) =>
		{
			io::stderr().write_line(fmt!("Error: expected %? but found error %s", expected, *failure.mesg));
			return false;
		}
	}
}

fn check_ok_str_arrays(result: status<@~[~str]>, expected: &[~str]) -> bool
{
	match result
	{
		result::Ok(pass) =>
		{
			if !vec::eq(*pass.value, expected)
			{
				io::stderr().write_line(fmt!("Expected %? but found %?", expected, pass.value));
				return false;
			}
			return true;
		}
		result::Err(failure) =>
		{
			io::stderr().write_line(fmt!("Error: expected %? but found error %s", expected, *failure.mesg));
			return false;
		}
	}
}

fn check_failed<T: copy owned>(result: status<T>, expected: &str, line: int) -> bool
{
	match result
	{
		result::Ok(pass) =>
		{
			io::stderr().write_line(fmt!("Expected error '%s' but found %?", expected.to_unique(), pass.value));
			return false;
		}
		result::Err(failure) =>
		{
			if failure.mesg != @expected.to_unique()
			{
				io::stderr().write_line(fmt!("Expected error '%s' but found error '%s'", expected.to_unique(), *failure.mesg));
				return false;
			}
			if failure.err_state.line != line
			{
				io::stderr().write_line(fmt!("Expected error '%s' on line %d but line is %d", expected.to_unique(), line, failure.err_state.line));
				return false;
			}
			return true;
		}
	}
}
