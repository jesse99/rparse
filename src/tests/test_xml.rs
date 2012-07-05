// Test for a simple DOM style XML parser. Note that this is not intended to be 
// standards compliant or even very useful. Instead it is designed to test a parser
// that returns objects instead of evaluating in-place.
import to_str::to_str;
import result::*;
import test_helpers::*;

type attribute = {name: str, value: str};

enum xml
{
	// element name, attributes, children, content
	xxml(str, [attribute]/~, [xml]/~, str)
}

impl of to_str for xml
{
	fn to_str() -> str
	{
		alt self
		{
			xxml(name, attributes, children, content)
			{
				let attrs = vec::map(attributes, |a| #fmt["%s=\"%s\"", a.name, a.value] );
				let childs = vec::map(children, |c| c.to_str() );
				if vec::len(attrs) > 0u
				{
					ret #fmt["<%s %s>%s%s</%s>", name, str::connect(attrs, " "), str::connect(childs, ""), content, name];
				}
				else
				{
					ret #fmt["<%s>%s%s</%s>", name, str::connect(childs, ""), content, name];
				}
			}
		}
	}
}

impl of to_str for attribute
{
	fn to_str() -> str
	{
		ret #fmt["%s = \"%s\"", self.name, self.value];
	}
}

fn check_xml_ok(inText: str, expected: str, parser: parser<xml>) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	alt parser({file: "unit test", text: text, index: 0u, line: 1,})
	{
		result::ok(pass)
		{
			check_ok(result::ok({new_state: pass.new_state, value: pass.value.to_str()}), expected)
		}
		result::err(failure)
		{
			check_ok(result::err(failure), expected)
		}
	}
}

fn check_xml_failed(inText: str, parser: parser<xml>, expected: str, line: int) -> bool
{
	#info["----------------------------------------------------"];
	let text = chars_with_eot(inText);
	let result = parser({file: "unit test", text: text, index: 0u, line: 1});
	ret check_failed(result, expected, line);
}

// string_body := [^"]*
fn string_body() -> parser<str>
{
	do scan0()
	|chars, i| {
		if chars[i] == '\\' && chars[i+1u] == '"'
		{
			2u
		}
		else if chars[i] != '"'
		{
			1u
		}
		else
		{
			0u
		}
	}
}

// content := (anything but '</')*
fn content() -> parser<str>
{
	do scan0()
	|chars, i| {
		if chars[i] == '<' && chars[i+1u] == '/'
		{
			0u
		}
		else
		{
			1u
		}
	}
}

fn xml_parser() -> parser<xml>
{
	let name = identifier().s0();
	
	let dummy = xxml("dummy", []/~, []/~, "");
	let element_ptr = @mut return(dummy);
	let element_ref = forward_ref(element_ptr);
	
	// attribute := name '=' '"' string_body '"'
	let attribute = do seq5(name, "=".s0(), "\"".s0(), string_body(), "\"".s0())
	|name, _a2, _a3, body, _a5| {
		result::ok({name: name, value: body})
	};
	
	// empty_element := '<' name attribute* '/>'
	let empty_element = do seq4("<".s0(), name, attribute.r0(), "/>".s0())
	|_a1, name, attrs, _a4| {
		result::ok(xxml(name, attrs, []/~, ""))
	};
	
	// complex_element := '<' name attribute* '>' element* content '</' name '>'
	let complex_element = do seq9("<".s0(), name, attribute.r0(), ">".s0(), element_ref.r0(), content(), "</".s0(), name, ">".s0())
	|_a1, name1, attrs, _a4, children, chars, _a5, name2, _a7| {
		if name1 == name2
		{
			result::ok(xxml(name1, attrs, children, chars))
		}
		else
		{
			result::err(#fmt["end tag '%s' but found '%s'", name1, name2])
		}
	};
	
	// element := empty_element | complex_element
	let element = empty_element.or(complex_element);
	*element_ptr = element;
	
	// start := s0 element EOT
	let s = return(dummy).s0();
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




