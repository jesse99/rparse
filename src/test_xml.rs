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
	nname(str),
	nattributes([attribute]),
	nxml(xml)
}

impl of to_str for xml
{
	fn to_str() -> str
	{
		alt self
		{
			xxml(name, attributes, children, content)
			{
				let attrs = vec::map(attributes) {|a| #fmt["%s = \"%s\" ", a.name, a.value]};
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

impl of to_str for node
{
	fn to_str() -> str
	{
		alt self
		{
			nname(name)
			{
				ret #fmt["name = %s", name];
			}
			nattributes(attributes)
			{
				let attrs = vec::map(attributes) {|a| #fmt["%s = \"%s\" ", a.name, a.value]};
				ret #fmt["attributes = %s", str::connect(attrs, " ")];
			}
			nxml(child)
			{
				ret child.to_str();
			}
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
				nname(name)
				{
					io::stderr().write_line(#fmt["Error: expected xml node but found name %s", name]);
					ret false;
				}
				nattributes(attrs)
				{
					io::stderr().write_line(#fmt["Error: expected xml node but found attributes %?", attrs]);
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
	ret plog("name", input, result::ok({value: nname(s) with answer}));
}

// element1 := '<' name attribute* '/>'
fn element1(input: state<node>) -> status<node>
{
	let s = space_zero_or_more(_);
	let lt = literal(_, "<", s);
	let slash_gt = literal(_, "/>", s);
	
	// TODO: parse attributes too
	let result = sequence(input, [lt, name, slash_gt])
	{
		|results|
		alt results[2]
		{
			nname(s)	{nxml(xxml(s, [], [], ""))}
			_			{fail "name should return nxml";}
		}
	};
	ret plog("element1", input, result);
}

// start := element
// element := element1 | element2
// element2 := '<' name attribute* '>' element* content '</' name '>'
// attribute := name '=' '"' [^"]* '"'
// content := <anything but '</'>*
fn xml_parser() -> str_parser<node>
{
	let s = space_zero_or_more(_);

	let element = element1(_);

	let start = everything("unit test", element, s, nxml(xxml("", [], [], "")), _);
	ret start;
}

#[test]
fn test_factor()
{
	let parser = xml_parser();
	
	assert xml_ok("<trivial/>", "<trivial></trivial>", parser);
	assert xml_ok("<simple></simple>", "<simple></simple>", parser);
	assert check_err_str("<simple></oops>", parser, "xxx", 1);
	
	// TODO:
	// attributes
	// child elements
	// content
}
