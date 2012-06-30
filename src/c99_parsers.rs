#[doc = "Functions that can be used to parse C99 lexical elements (or with languages
that have similar lexical elements)."];

// See http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1539.pdf
export identifier, decimal_number, octal_number, hex_number, float_number,
	char_literal, string_literal, comment, line_comment;

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

float1 := [0-9]* '.' [0-9]+ exponent?
float2 := [0-9]+ '.' exponent?
float3 := [0-9]+ exponent
exponent := [eE] [+-]? [0-9]+"]
fn float_number() -> parser<f64>
{
	let exponent = seq3_ret_str("eE".anyc(), "+-".anyc().optional(), match1(is_digit));
	
	let float1 = seq4_ret_str(match0(is_digit), ".".lit(), match1(is_digit), exponent.optional()).tag("");
	let float2 = seq3_ret_str(match1(is_digit), ".".lit(), exponent.optional()).tag("");
	let float3 = seq2_ret_str(match1(is_digit), exponent).tag("");
	
	let number = or_v([float1, float2, float3]/~).tag("Expected float number");
	
	number.thene()
		{|text|
			str::as_c_str(text)
			{|ptr|
				return(libc::strtod(ptr, ptr::null()) as f64)
			}
		}
}

#[doc = "char_literal := '\\'' c_char_sequence '\\''

c_char_sequence := [^'\n\r\\]
c_char_sequence := escape_sequence"]
fn char_literal() -> parser<char>
{
	// We don't support the [LuU] prefix (so the parser is reusable in other contexts).
	let case1 = "'\n\r\\".noc().tag("");
	let case2 = escape_sequence().tag("Expected escape character");
	let char_sequence = case1.or(case2);
	
	seq3_ret1("'".lit(), char_sequence, "'".lit()).tag("Expected char literal")
}

#[doc = "string_literal := '\"' s_char* '\"'

s_char := [^\"\n\r\\]
s_char := escape_sequence"]
fn string_literal() -> parser<str>
{
	// We don't support the encoding prefix (so the parser is reusable in other contexts).
	let case1 = "\"\n\r\\".noc().tag("");
	let case2 = escape_sequence().tag("Expected escape character");
	let s_char = case1.or(case2);
	let body = s_char.r0().thene() {|chars| return(str::from_chars(chars))};
	
	seq3_ret1("\"".lit(), body, "\"".lit()).tag("Expected string literal")
}

#[doc = "comment := '/*' ([^*] | '*' [^/])* '*/'

Note that these do not nest."]
fn comment() -> parser<str>
{
	let body = scan0()
	{|chars, i|
		if chars[i] == '*' && chars[i+1u] == '/'
		{
			0u
		}
		else
		{
			1u
		}
	};
	
	seq3_ret1("/*".lit(), body, "*/".lit()).tag("Expected comment")
}

#[doc = "line_comment := '//' [^\r\n]*"]
fn line_comment() -> parser<str>
{
	let body = scan0()
	{|chars, i|
		if chars[i] == '\r' || chars[i+1u] == '\n'
		{
			0u
		}
		else
		{
			1u
		}
	};
	
	seq2_ret1("//".lit(), body).tag("Expected line comment")
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

fn escape_to_char(ch: char) -> char
{
	alt ch
	{
		'a'
		{
			'\x07'
		}
		'b'
		{
			'\x7F'
		}
		'f'
		{
			'\x0C'
		}
		'n'
		{
			'\n'
		}
		'r'
		{
			'\r'
		}
		't'
		{
			'\t'
		}
		'v'
		{
			'\x0B'
		}
		_
		{
			ch
		}
	}
}

fn octal_digits() -> parser<int>
{
	match1(is_octal).tag("Expected octal escape").thene()
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

fn hex_digits() -> parser<int>
{
	match1(is_hex).tag("Expected hex escape").thene()
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
		}
}

// escape-sequence := '\\' ['"?abfnrtv\\]
// escape-sequence := '\\' octal-digit{1, 3}
// escape-sequence := '\\x' hex-digit{1, 2}
// escape-sequence := universal-character-name
fn escape_sequence() -> parser<char>
{
	let escape = "'\"?abfnrtv\\".anyc().thene()
		{|ch| return(escape_to_char(ch))};
	
	let case1 = seq2_ret1("\\".lit(), escape);
	let case2 = seq2_ret1("\\".lit(), octal_digits().thene({|n| return(n as char)}));
	let case3 = seq2_ret1("\\x".lit(), hex_digits().thene({|n| return(n as char)}));
	let case4 = universal_character_name();
	or_v([case1, case2, case3, case4]/~).tag("")
}

// universal-character-name := '\\u' hex-digit{4}
// universal-character-name := '\\U' hex-digit{8}
fn universal_character_name() -> parser<char>
{
	seq3_ret2("\\".lit(), "uU".anyc(), hex_digits()).tag("Expected unicode escape").thene({|n| return(n as char)})
}


