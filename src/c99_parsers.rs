//! Functions that can be used to parse C99 lexical elements (or with languages
//! that have similar lexical elements).

// See http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1539.pdf

/// identifier := [a-zA-Z_] [a-zA-Z0-9_]*
/// 
/// Note that match1_0 can be used to easily implement custom identifier parsers.
pub fn identifier() -> Parser<@~str>
{
	// Supposed to support universal character names too, e.g.  
	// fo\u006F is a valid C99 identifier.
	match1_0(is_identifier_prefix, is_identifier_suffix)
}

/// decimal_number := [0-9]+
/// 
/// Technically this is not supposed to match numbers with leading zeros,
/// but we do so to make this parser more reusable.
pub fn decimal_number() -> Parser<int>
{
	do match1(is_digit).thene
		|text|
		{
			match int::from_str(*text)
			{
				option::Some(value) =>
				{
					ret(value)
				}
				_ =>
				{
					fails(fmt!("'%s' is out of range", *text))
				}
			}
		}
}

/// octal_number := 0 [0-7]*
pub fn octal_number() -> Parser<int>
{
	do match1_0(|c| c == '0', is_octal).thene
		|text|
		{
			match from_base_8(*text)
			{
				result::Ok(value) =>
				{
					ret(value)
				}
				result::Err(copy mesg) =>
				{
					fails(mesg)
				}
			}
		}
}

/// hex_number := 0[xX] [0-9a-fA-F]+
pub fn hex_number() -> Parser<int>
{
	let prefix = "0".lit().then("x".lit().or("X".lit()));
	let digits = do match1(is_hex).thene()
			|text| {
			match from_base_16(*text)
			{
				result::Ok(value) =>
				{
					ret(value)
				}
				result::Err(copy mesg) =>
				{
					fails(mesg)
				}
			}
		};
	
	seq2_ret1(prefix, digits)
}

/// float_number := float1 | float2 | float3
/// 
/// float1 := [0-9]* '.' [0-9]+ exponent?
/// float2 := [0-9]+ '.' exponent?
/// float3 := [0-9]+ exponent
/// exponent := [eE] [+-]? [0-9]+
pub fn float_number() -> Parser<f64>
{
	let exponent = seq3_ret_str("eE".anyc(), "+-".anyc().optional(), match1(is_digit));
	
	let float1 = seq4_ret_str(match0(is_digit), ".".lit(), match1(is_digit), exponent.optional()).err("");
	let float2 = seq3_ret_str(match1(is_digit), ".".lit(), exponent.optional()).err("");
	let float3 = seq2_ret_str(match1(is_digit), exponent).err("");
	
	let number = or_v(@~[float1, float2, float3]);
	
	do number.thene()
		|text| {
                        do str::as_c_str(*text)
                        |ptr| {
				ret(libc::strtod(ptr, ptr::null()) as f64)
			}
		}
}

/// char_literal := '\\'' c_char_sequence '\\''
/// 
/// c_char_sequence := [^'\n\r\\]
/// c_char_sequence := escape_sequence
pub fn char_literal() -> Parser<char>
{
	// We don't support the [LuU] prefix (so the parser is reusable in other contexts).
	let case1 = "'\n\r\\".noc().err("");
	let case2 = escape_sequence().err("escape character");
	let char_sequence = case1.or(case2);
	
	seq3_ret1("'".lit(), char_sequence, "'".lit())
}

/// string_literal := '\"' s_char* '\"'
/// 
/// s_char := [^\"\n\r\\]
/// s_char := escape_sequence
pub fn string_literal() -> Parser<@~str>
{
	// We don't support the encoding prefix (so the parser is reusable in other contexts).
	let case1 = "\"\n\r\\".noc().err("");
	let case2 = escape_sequence().err("escape character");
	let s_char = case1.or(case2);
	let body = do s_char.r0().thene() |chars| { ret(@str::from_chars(*chars))};
	
	seq3_ret1("\"".lit(), body, "\"".lit())
}

/// comment := '/*' ([^*] | '*' [^/])* '*/'
/// 
/// Note that these do not nest.
pub fn comment() -> Parser<@~str>
{
	fn comment_body(chars: @[char], index: uint) -> uint
	{
		let mut i = index;
		loop
		{
			if chars[i] == EOT
			{
				return 0;
			}
			else if chars[i] == '*' && chars[i+1] == '/'
			{
				return i - index;
			}
			else
			{
				i += 1;
			}
		}
	}
	
	let body = scan(comment_body);
	seq3_ret1("/*".lit(), body, "*/".lit())
}

/// line_comment := '//' [^\r\n]*
pub fn line_comment() -> Parser<@~str>
{
	fn comment_body(chars: @[char], index: uint) -> uint
	{
		let mut i = index;
		loop
		{
			if chars[i] == '\r' || chars[i] == '\n' || chars[i] == EOT
			{
				return i - index;
			}
			else
			{
				i += 1;
			}
		}
	}
	
	let body = scan(comment_body);
	seq2_ret1("//".lit(), body)
}

// ---- Helpers ---------------------------------------------------------------
pure fn is_identifier_prefix(ch: char) -> bool
{
	return is_alpha(ch) || ch == '_';
}

pure fn is_identifier_suffix(ch: char) -> bool
{
	return is_identifier_prefix(ch) || is_digit(ch);
}

pure fn is_octal(ch: char) -> bool
{
	return ch >= '0' && ch <= '7';
}

pure fn is_hex(ch: char) -> bool
{
	return (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F');
}

fn from_base_8(text: &str) -> result::Result<int, ~str>
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
			return result::Err(~"Octal number is too large");
		}
		power *= 8;
	}
	
	return result::Ok(result);
}

fn from_base_16(text: &str) -> result::Result<int, ~str>
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
			return result::Err(~"Hex number is too large");
		}
		power *= 16;
	}
	
	return result::Ok(result);
}

fn escape_to_char(ch: char) -> char
{
	match ch
	{
		'a' =>
		{
			'\x07'
		}
		'b' =>
		{
			'\x7F'
		}
		'f' =>
		{
			'\x0C'
		}
		'n' =>
		{
			'\n'
		}
		'r' =>
		{
			'\r'
		}
		't' =>
		{
			'\t'
		}
		'v' =>
		{
			'\x0B'
		}
		_ =>
		{
			ch
		}
	}
}

fn octal_digits() -> Parser<int>
{
	do match1(is_octal).thene()
		    |text| {
			match from_base_8(*text)
			{
				result::Ok(value) =>
				{
					ret(value)
				}
				result::Err(copy mesg) =>
				{
					fails(mesg)
				}
			}
		}
}

fn hex_digits() -> Parser<int>
{
	do match1(is_hex).thene()
		|text| {
			match from_base_16(*text)
			{
				result::Ok(value) =>
				{
					ret(value)
				}
				result::Err(copy mesg) =>
				{
					fails(mesg)
				}
			}
		}
}

// escape-sequence := '\\' ['"?abfnrtv\\]
// escape-sequence := '\\' octal-digit{1, 3}
// escape-sequence := '\\x' hex-digit{1, 2}
// escape-sequence := universal-character-name
fn escape_sequence() -> Parser<char>
{
	let escape = do "'\"?abfnrtv\\".anyc().thene()
		|ch| {ret(escape_to_char(ch))};
	
	let case1 = seq2_ret1("\\".lit(), escape);
	let case2 = seq2_ret1("\\".lit(), octal_digits().thene(|n| ret(n as char) ));
	let case3 = seq2_ret1("\\x".lit(), hex_digits().thene(|n| ret(n as char) ));
	let case4 = universal_character_name();
	or_v(@~[case1, case2, case3, case4]).err("")
}

// universal-character-name := '\\u' hex-digit{4}
// universal-character-name := '\\U' hex-digit{8}
fn universal_character_name() -> Parser<char>
{
	seq3_ret2("\\".lit(), "uU".anyc(), hex_digits()).thene(|n| ret(n as char) )
}


