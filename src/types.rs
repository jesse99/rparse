#[doc = "The argument and return types used by all parse functions."];

import result = result::result;

#[doc = "Input argument for parse functions. File is not interpreted 
and need not be a path. Text is assumed to end with EOT. Lines are 1-based."]
type state = {file: str, text: [char], index: uint, line: int};


#[doc = "Return type of parse functions."]
type status<T> = result<passed<T>, failed>;

#[doc = "Included in the result of parse functions which succeeded."]
type passed<T> = {output: state, value: T};

#[doc = "Included in the result of parse functions which failed.
Output will be the same as the input state."]
type failed = {output: state, mesg: str};


#[doc = "Type for parse functions."]
type parser<T> = fn (state) -> status<T>;
