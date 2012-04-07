#[doc = "The argument and return types used by all parse functions."];

import result = result::result;

#[doc = "Return type of parse functions."]
type status<T: copy> = result<state<T>, failed<T>>;

#[doc = "Input argument for parse functions. File is not interpreted 
and need not be a path. Text is assumed to end with EOT. Lines are 1-based.
Value is the current result of the parse."]
type state<T: copy> = {file: str, text: [char], index: uint, line: int, value: T};

#[doc = "Included in the result of parse functions which failed.
Output will be the same as the input state."]
type failed<T: copy> = {output: state<T>, mesg: str};


#[doc = "Type for parse functions."]
type parser<T: copy> = fn@ (state<T>) -> status<T>;

#[doc = "Type for top-level parse functions."]
type str_parser<T: copy> = fn@ (str) -> status<T>;
