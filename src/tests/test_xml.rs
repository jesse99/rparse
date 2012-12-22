// Test for a simple DOM style XML parser. Note that this is not intended to be 
// standards compliant or even very useful. Instead it is designed to test a parser
// that returns objects instead of evaluating in-place.
//use to_str::to_str;
use c99_parsers::*;
use parsers::*;
use result::*;
use tests::test_helpers::*;

struct Attribute {name: @~str, value: @~str}

enum Xml
{
	// element name, attributes, children, content
	XXml(@~str, @~[Attribute], @~[Xml], @~str)
}

impl  Xml : ToStr 
{
	pure fn to_str() -> ~str
	{
		match self
		{
			XXml(name, attributes, children, content) =>
			{
				let attrs = vec::map(*attributes, |a| fmt!("%s=\"%s\"", *a.name, *a.value) );
				let childs = vec::map(*children, |c| c.to_str() );
				if vec::len(attrs) > 0u
				{
					return fmt!("<%s %s>%s%s</%s>", *name, str::connect(attrs, ~" "), str::connect(childs, ~""), *content, *name);
				}
				else
				{
					return fmt!("<%s>%s%s</%s>", *name, str::connect(childs, ~""), *content, *name);
				}
			}
		}
	}
}

impl  Attribute : ToStr 
{
	pure fn to_str() -> ~str
	{
		return fmt!("%s = \"%s\"", *self.name, *self.value);
	}
}

fn check_xml_ok(inText: &str, expected: &str, parser: Parser<Xml>) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	match parser(State {file: @~"unit test", text: text, index: 0u, line: 1,})
	{
		result::Ok(ref pass) =>
		{
			check_ok(&result::Ok(Succeeded {new_state: pass.new_state, value: @pass.value.to_str()}), &@expected.to_owned())
		}
		result::Err(ref failure) =>
		{
			check_ok_strs(&result::Err(*failure), expected)
		}
	}
}

fn check_xml_failed(inText: &str, parser: Parser<Xml>, expected: &str, line: int) -> bool
{
	info!("----------------------------------------------------");
	let text = chars_with_eot(inText);
	let result = parser(State {file: @~"unit test", text: text, index: 0u, line: 1});
	return check_failed(&result, expected, line);
}

// string_body := [^"]*
fn string_body() -> Parser<@~str>
{
	fn body(chars: @[char], index: uint) -> uint
	{
		let mut i = index;
		loop
		{
			if chars[i] == EOT
			{
				return 0;
			}
			else if chars[i] == '\\' && chars[i+1] == '"'
			{
				i += 2;
			}
			else if chars[i] != '"'
			{
				i += 1;
			}
			else
			{
				return i - index;
			}
		}
	}
	
	scan(body)
}

// content := (anything but '</')*
fn content() -> Parser<@~str>
{
	fn body(chars: @[char], index: uint) -> uint
	{
		let mut i = index;
		loop
		{
			if chars[i] == EOT
			{
				return 0;
			}
			else if chars[i] == '<' && chars[i+1u] == '/'
			{
				return i - index;
			}
			else
			{
				i += 1;
			}
		}
	}
	
	scan(body)
}

fn xml_parser() -> Parser<Xml>
{
	let name = identifier().s0();
	
	let dummy = XXml(@~"dummy", @~[], @~[], @~"");
	let element_ptr = @mut ret(dummy);
	let element_ref = forward_ref(element_ptr); 
	
	// attribute := name '=' '"' string_body '"'
	let attribute = do seq5(name, "=".s0(), "\"".s0(), string_body(), "\"".s0())
	|name, _a2, _a3, body, _a5| {
		result::Ok(Attribute {name: name, value: body})
	};
	
	// empty_element := '<' name attribute* '/>'
	let empty_element = do seq4("<".s0(), name, attribute.r0(), "/>".s0())
	|_a1, name, attrs, _a4| {
		result::Ok(XXml(name, attrs, @~[], @~""))
	};
	
	// complex_element := '<' name attribute* '>' element* content '</' name '>'
	let complex_element = do seq9("<".s0(), name, attribute.r0(), ">".s0(), element_ref.r0(), content(), "</".s0(), name, ">".s0())
	|_a1, name1, attrs, _a4, children, chars, _a5, name2, _a7|
	{
		if name1 == name2
		{
			result::Ok(XXml(name1, attrs, children, chars))
		}
		else
		{
			result::Err(@fmt!("end tag '%s' but found '%s'", *name1, *name2))
		}
	};
	
	// element := empty_element | complex_element
	let element = empty_element.or(complex_element);
	*element_ptr = element;
	
	// start := s0 element EOT
	let s = ret(dummy).s0();
	element.everything(s)
}

#[test]
fn test_simple_element()
{
	let p = xml_parser();
	
	assert check_xml_ok("<trivial/>", "<trivial></trivial>", p);
	assert check_xml_ok("<trivial first=\"number one\"/>", "<trivial first=\"number one\"></trivial>", p);
	assert check_xml_ok("<trivial first=\"number one\" second=\"number two\"/>", "<trivial first=\"number one\" second=\"number two\"></trivial>", p);
	assert check_xml_ok("  <  trivial first \t =    \"number one\"  \t/>", "<trivial first=\"number one\"></trivial>", p);
	assert check_xml_failed("<trivial", p, "'/>' or '>'", 1);
	assert check_xml_failed("<trivial first=\"number one/>", p, "'/>' or '>'", 1);
}

#[test]
fn test_element()
{
	let p = xml_parser();
	
	assert check_xml_ok("<simple>\n  \n</simple>", "<simple></simple>", p);
	assert check_xml_failed("<simple></oops>", p, "end tag 'simple' but found 'oops'", 1);
	assert check_xml_ok("<simple alpha = \"A\" beta=\"12\"></simple>", "<simple alpha=\"A\" beta=\"12\"></simple>", p);
	
	assert check_xml_ok("<parent><child></child></parent>", "<parent><child></child></parent>", p);
	assert check_xml_ok("<parent><child/></parent>", "<parent><child></child></parent>", p);
	assert check_xml_ok("<parent><child1/><child2/></parent>", "<parent><child1></child1><child2></child2></parent>", p);
	assert check_xml_ok("<parent><child1><child2></child2></child1></parent>", "<parent><child1><child2></child2></child1></parent>", p);
	
	assert check_xml_ok("<parent>some text</parent>", "<parent>some text</parent>", p);
	assert check_xml_ok("<parent><child/>blah blah</parent>", "<parent><child></child>blah blah</parent>", p);
	assert check_xml_failed("<simple>\r  \n  \r\n</oops>", p, "end tag 'simple' but found 'oops'", 4);
}

#[testXX]
fn test_x()
{
	let p = xml_parser();
	assert check_xml_ok("<trivial/>", "<trivial></trivial>", p);
}




