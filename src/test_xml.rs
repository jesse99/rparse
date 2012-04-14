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
	xxml(str, [attribute], [xml], str)
}

// Parse tree node, i.e. intermediate data structure which will eventually become an xml object.
enum node
{
	nname(str),
	nchildren([xml]),
	nattribute(attribute),
	nattributes([attribute]),
	nxml(xml),
	ntext(str)
}

fn make_attribute_node(name: node, value: node) -> node
{
	let n = alt name {nname(x) {x} 	_ {fail}};
	let v = alt value {ntext(x) {x}	_ {fail}};
	ret nattribute({name: n, value: v});
}

fn make_attributes_node(attrs: [node]) -> node
{
	let a = vec::map(attrs, {|v| alt v {nattribute(x) {x} _ {fail}}});
	ret nattributes(a);
}

fn make_children_node(children: [node]) -> node
{
	let c = vec::map(children, {|v| alt v {nxml(x) {x} _ {fail}}});
	ret nchildren(c);
}

fn make_simple_node(name: node, attributes: node) -> node
{
	let n = alt name {nname(x) {x} 				_ {fail}};
	let a = alt attributes {nattributes(x) {x}	_ {fail}};
	ret nxml(xxml(n, a, [], ""));
}

fn make_complex_node(name: node, attributes: node, children: node, content: node) -> node
{
	let n = alt name {nname(x) {x} 				_ {fail}};
	let a = alt attributes {nattributes(x) {x}	_ {fail}};
	let c = alt children {nchildren(x) {x}		_ {fail}};
	let t = alt content {ntext(x) {x}				_ {fail}};
	ret nxml(xxml(n, a, c, t));
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
			nname(s)			{ret s;}
			nchildren(cc)		{let children = vec::map(cc) {|c| c.to_str()}; ret str::connect(children, "");}
			nattribute(a)		{ret #fmt["%s=\"%s\"", a.name, a.value];}
			nattributes(aa)	{let attrs = vec::map(aa) {|a| #fmt["%s=\"%s\"", a.name, a.value]}; ret str::connect(attrs, " ");}
			nxml(x)			{ret x.to_str();}
			ntext(s)			{ret s;}
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
				_
				{
					io::stderr().write_line(#fmt["Error: expected xml node but found attribute %s", answer.value.to_str()]);
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

fn string(input: state<node>) -> status<node>
{
	let start = input.index;
	let mut i = start;
	while input.text[i] != '"' && input.text[i] != EOT
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
		|values|
		result::ok(make_attribute_node(values[0], values[3]))
	};
	ret plog("attribute", input, result);
}

// attributes := attribute*
fn attributes(input: state<node>) -> status<node>
{
	let result = repeat_zero_or_more(input, attribute(_))
	{
		|values|
		result::ok(make_attributes_node(values))
	};
	ret plog("attributes", input, result);
}

// content := (anything but '</')*
fn content(input: state<node>) -> status<node>
{
	let mut i = input.index;
	while !(input.text[i] == '<' && input.text[i+1u] == '/') && input.text[i] != EOT
	{
		// TODO: need to increment line number on EOL characters
		i += 1u;
	}
	
	if input.text[i] == '<' && input.text[i+1u] == '/'
	{
		let text = str::from_chars(vec::slice(input.text, input.index, i));
		ret plog("content", input, result::ok({index: i, value: ntext(text) with input}));
	}
	else
	{
		ret plog("content", input, result::err({output: input, maxIndex: i, mesg: "element content should be followed by an end tag"}));
	}
}

// empty_element := '<' name attribute* '/>'
fn empty_element(input: state<node>) -> status<node>
{
	let s = space_zero_or_more(_);
	let lt = literal(_, "<", s);
	let slash_gt = literal(_, "/>", s);
	
	let result = sequence(input, [lt, name, attributes(_), slash_gt])
	{
		|values|
		result::ok(make_simple_node(values[1], values[2]))
	};
	ret plog("empty_element", input, result);
}

// complex_element := '<' name attribute* '>' element* content '</' name '>'
fn complex_element(input: state<node>, element_ptr: @mut parser<node>) -> status<node>
{
	let s = space_zero_or_more(_);
	let lt = literal(_, "<", s);
	let gt = literal(_, ">", s);
	let lt_slash = literal(_, "</", s);
	let children = repeat_zero_or_more(_, forward_ref(_, element_ptr),
	{
		|values|
		result::ok(make_children_node(values))
	});
	let body = content(_);
	
	//                                        0  1        2                  3   4           5        6          7        8
	let result = sequence(input, [lt, name, attributes(_), gt, children, body, lt_slash, name, gt])
	{
		|values|
		let name1 = values[1].to_str();
		let name2 = values[7].to_str();
		if name1 == name2
		{
			result::ok(make_complex_node(values[1], values[2], values[4], values[5]))
		}
		else
		{
			result::err(#fmt["Expected end tag '%s' but found '%s'", name1, name2])
		}
	};
	ret plog("complex_element", input, result);
}

fn xml_parser() -> str_parser<node>
{
	let s = space_zero_or_more(_);
	
	// element := empty_element | complex_element
	let element_ptr = @mut fails(_);
	let element = alternative(_, [empty_element(_), complex_element(_, element_ptr)]);
	*element_ptr = element;
	
	// start := element
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
	assert check_err_str("<simple></oops>", parser, "Expected end tag 'simple' but found 'oops'", 1);
	assert xml_ok("<simple alpha = \"A\" beta=\"12\"></simple>", "<simple alpha=\"A\" beta=\"12\"></simple>", parser);
	
	assert xml_ok("<parent><child></child></parent>", "<parent><child></child></parent>", parser);
	assert xml_ok("<parent><child/></parent>", "<parent><child></child></parent>", parser);
	assert xml_ok("<parent><child1/><child2/></parent>", "<parent><child1></child1><child2></child2></parent>", parser);
	assert xml_ok("<parent><child1><child2></child2></child1></parent>", "<parent><child1><child2></child2></child1></parent>", parser);
	
	assert xml_ok("<parent>some text</parent>", "<parent>some text</parent>", parser);
	assert xml_ok("<parent><child/>blah blah</parent>", "<parent><child></child>blah blah</parent>", parser);
}

// TODO:
// may want to use constructors like name_node
// check (some) funky whitespace
// check some more error cases
// check a real xml example
// probably want to get rid of binary_op
//    or move it into test_expr

