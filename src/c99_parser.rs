#[doc = "Functions that can be used to parse C99 lexical elements (or with languages
that have similar lexical elements)."];

// See http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1539.pdf
export identifier, decimal_number, octal_number, hex_number, float_number;

#[doc = "identifier := [a-zA-Z_] [a-zA-Z0-9_]*

Note that match1_0 can be used to easily implement custom identifier parsers."]
fn identifier() -> parser<str>
{
	// Supposed to support universal character names too, e.g.
	// fo\u006F is a valid C99 identifier.
	match1_0(is_identifier_prefix, is_identifier_suffix).tag("Expected identifier")
}

#[doc = "decimal_number := [0-9]+

Technically this is not supposed to match numbers with leading zeros,
but we do so to make this parser more reusable."]
fn decimal_number() -> parser<int>
{
	match1(is_digit).tag("Expected decimal number").thene()
		{|text|
			alt int::from_str(text)
			{
				option::some(value)
				{
					return(value)
				}
				_
				{
					fails(#fmt["'%s' is out of range", text])
				}
			}
		}
}

#[doc = "octal_number := 0 [0-7]*"]
fn octal_number() -> parser<int>
{
	match1_0({|c| c == '0'}, is_octal).tag("Expected octal number").thene()
		{|text|
			alt from_base_8(text)
			{
				result::ok(value)
				{
					return(value)
				}
				result::err(mesg)
				{
					fails(mesg)
				}
			}
		}
}

#[doc = "hex_number := 0[xX] [0-9a-fA-F]+"]
fn hex_number() -> parser<int>
{
	let prefix = "0".lit().then(or("x".lit(), "X".lit())).tag("Expected hex number");
	let digits = match1(is_hex).tag("Expected hex number").thene()
		{|text|
			alt from_base_16(text)
			{
				result::ok(value)
				{
					return(value)
				}
				result::err(mesg)
				{
					fails(mesg)
				}
			}
		};
	
	seq2_ret1(prefix, digits)
}

#[doc = "float_number := float1 | float2 | float3

// float1 := [0-9]* '.' [0-9]+ exponent?
// float2 := [0-9]+ '.' exponent?
// float3 := [0-9]+ exponent
// exponent := [eE] [+-]? [0-9]+"]
fn float_number() -> parser<f64>
{
	let exponent = seq3_ret_str("eE".anyc(), "+-".anyc().optional(), match1(is_digit));
	
	let float1 = seq4_ret_str(match0(is_digit), ".".lit(), match1(is_digit), exponent.optional()).tag("");
	let float2 = seq3_ret_str(match1(is_digit), ".".lit(), exponent.optional()).tag("");
	let float3 = seq2_ret_str(match1(is_digit), exponent).tag("");
	
	let number = or_v([float1, float2, float3]).tag("Expected float number");
	
	number.thene()
		{|text|
			str::as_c_str(text)
			{|ptr|
				return(libc::strtod(ptr, ptr::null()) as f64)
			}
		}
}

// ---- Helpers ---------------------------------------------------------------
pure fn is_identifier_prefix(ch: char) -> bool
{
	ret is_alpha(ch) || ch == '_';
}

pure fn is_identifier_suffix(ch: char) -> bool
{
	ret is_identifier_prefix(ch) || is_digit(ch);
}

pure fn is_octal(ch: char) -> bool
{
	ret ch >= '0' && ch <= '7';
}

pure fn is_hex(ch: char) -> bool
{
	ret (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F');
}

fn from_base_8(text: str) -> result::result<int, str>
{
	let mut power = 1;
	let mut result = 0;
	
	let mut i = str::len(text);
	while i > 0u
	{
		i -= 1u;
		
		let delta = (text[i] - ('0' as u8)) as int;
		if power*delta < int::max_value - result
		{
			result += power*delta;
		}
		else
		{
			ret result::err("Octal number is too large");
		}
		power *= 8;
	}
	
	ret result::ok(result);
}

fn from_base_16(text: str) -> result::result<int, str>
{
	let mut power = 1;
	let mut result = 0;
	
	let mut i = str::len(text);
	while i > 0u
	{
		i -= 1u;
		
		let ch = text[i] as char;
		let delta = 
			if ch >= '0' && ch <= '9'
			{
				(text[i] - ('0' as u8)) as int
			}
			else if ch >= 'a' && ch <= 'f'
			{
				10 + (text[i] - ('a' as u8)) as int
			}
			else
			{
				10 + (text[i] - ('A' as u8)) as int
			};
		if power*delta < int::max_value - result
		{
			result += power*delta;
		}
		else
		{
			ret result::err("Hex number is too large");
		}
		power *= 16;
	}
	
	ret result::ok(result);
}

// char_literal
// string_literal
// comment				these don't nest
// line_comment
