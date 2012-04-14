// Test for a simple DOM style XML parser. Note that this is not intended to be 
// standards compliant or even very useful. Instead it is designed to test a parser
// returning a parse tree sort of thing.
import io;
import io::writer_util;
import to_str::to_str;
import result::*;
import test_helpers::*;

type attribute = {name: str, value: str};

enum xml
{
	// element name, attributes, children, content
	xxml(str, [attribute], [@xml], str)
}

enum node
{
	ntext(str),
	nattribute(attribute),
	nxml(xml)
}

fn get_node_text(node: node) -> str
{
	alt node
	{
		ntext(t)	{ret t;}
		_			{fail "expected ntext, but found " + node.to_str();}
	}
}

fn get_node_attr(node: node) -> attribute
{
	alt node
	{
		nattribute(a)	{ret a;}
		_				{fail "expected nattribute, but found " + node.to_str();}
	}
}

fn get_node_xml(node: node) -> xml
{
	alt node
	{
		nxml(x)	{ret x;}
		_			{fail "expected nxml, but found " + node.to_str();}
	}
}

impl of to_str for xml
{
	fn to_str() -> str
	{
		alt self
		{
			xxml(name, attributes, children, content)
			{
				let attrs = vec::map(attributes) {|a| #fmt["%s=\"%s\"", a.name, a.value]};
				let childs = vec::map(children) {|c| c.to_str()};
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

impl of to_str for node
{
	fn to_str() -> str
	{
		alt self
		{
			ntext(name)		{ret name;}
			nattribute(attr)	{ret #fmt["%s = \"%s\"", attr.name, attr.value];}
			nxml(child)		{ret child.to_str();}
		}
	}
}

#[cfg(test)]
fn xml_ok(text: str, expected: str, parser: str_parser<node>) -> bool
{
	alt parser(text)
	{
		result::ok(answer)
		{
			alt answer.value
			{
				nxml(child)
				{
					let actual = child.to_str();
					if actual != expected
					{
						io::stderr().write_line(#fmt["Expected %s but found %s", expected, actual]);
						ret false;
					}
					ret true;
				}
				ntext(name)
				{
					io::stderr().write_line(#fmt["Error: expected xml node but found name %s", name]);
					ret false;
				}
				nattribute(attr)
				{
					io::stderr().write_line(#fmt["Error: expected xml node but found attribute %?", attr.to_str()]);
					ret false;
				}
			}
		}
		result::err(error)
		{
			io::stderr().write_line(#fmt["Error: %s", error.mesg]);
			ret false;
		}
	}
}

fn name(input: state<node>) -> status<node>
{
	if !is_alpha(input.text[input.index])
	{
		ret plog("name", input, result::err({output: input, maxIndex: input.index, mesg: "expected an element name"}));
	}
	
	let start = input.index;
	let mut i = start;
	while is_alphanum(input.text[i]) || input.text[i] == '_'
	{
		i += 1u;
	}
	
	let s = str::from_chars(vec::slice(input.text, start, i));
	let answer = get(space_zero_or_more({index: i with input}));
	ret plog("name", input, result::ok({value: ntext(s) with answer}));
}

fn string(input: state<node>) -> status<node>
{
	let start = input.index;
	let mut i = start;
	while input.text[i] != '"'
	{
		i += 1u;
	}
	
	let s = str::from_chars(vec::slice(input.text, start, i));
	let answer = get(space_zero_or_more({index: i with input}));
	ret plog("string", input, result::ok({value: ntext(s) with answer}));
}

// attribute := name '=' '"' [^"]* '"'
fn attribute(input: state<node>) -> status<node>
{
	let s = space_zero_or_more(_);
	let eq = literal(_, "=", s);
	let quote = literal(_, "\"", s);
	let result = sequence(input, [name, eq, quote, string, quote])
	{
		|results|
		let name = get_node_text(results[1]);
		let value = get_node_text(results[4]);
		nattribute({name: name, value: value})
	};
	ret result;
}

fn add_attribute(node: xml, attr: attribute) -> node
{
	alt node
	{
		xxml(name, attrs, children, content)
		{
			nxml(xxml(name, attrs + [attr], children, content))
		}
	}
}

// attributes := attribute*
fn attributes(input: state<node>) -> status<node>
{
	let result = repeat_zero_or_more(input, attribute(_))
	{
		|lhs, rhs|
		let attr = get_node_attr(rhs);
		
		alt lhs
		{
			ntext(name)	{nxml(xxml(name, [attr], [], ""))}	// lhs was an element name
			nxml(child)	{add_attribute(child, attr)}			// lhs was an attribute
			_				{fail "attribute should be preceded by a name or an attribute";}
		}
	};
	ret plog("attributes", input, result);
}

// empty_element := '<' name attribute* '/>'
fn empty_element(input: state<node>) -> status<node>
{
	let s = space_zero_or_more(_);
	let lt = literal(_, "<", s);
	let slash_gt = literal(_, "/>", s);
	
	let result = sequence(input, [lt, name, attributes(_), slash_gt])
	{
		|results|
		alt results[3]
		{
			ntext(name)	{nxml(xxml(name, [], [], ""))}	// no attributes
			_				{results[3]}							// had attributes
		}
	};
	ret plog("empty_element", input, result);
}

// start := element
// element := empty_element | element
// element := '<' name attribute* '>' element* content '</' name '>'
// content := <anything but '</'>*
fn xml_parser() -> str_parser<node>
{
	let s = space_zero_or_more(_);

	let element = empty_element(_);

	let start = everything("unit test", element, s, nxml(xxml("", [], [], "")), _);
	ret start;
}

#[test]
fn test_simple_element()
{
	let parser = xml_parser();
	
	assert xml_ok("<trivial/>", "<trivial></trivial>", parser);
	assert xml_ok("<trivial first=\"number one\"/>", "<trivial first=\"number one\"></trivial>", parser);
	assert xml_ok("<trivial first=\"number one\" second=\"number two\"/>", "<trivial first=\"number one\" second=\"number two\"></trivial>", parser);
}

#[test]
fn test_element()
{
	let parser = xml_parser();
	
	assert xml_ok("<simple></simple>", "<simple></simple>", parser);
	assert check_err_str("<simple></oops>", parser, "xxx", 1);
}

// TODO:
// element
// check (some) funky whitespace
// attributes in element
// child elements
// content
// probably want to get rid of binary_op
