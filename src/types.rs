#[doc = "The types used by all parse functions."];

import result = result::result;

#[doc = "Type for parse functions."]
type parser<T: copy> = fn@ (state) -> status<T>;


#[doc = "Input argument for parse functions. File is not interpreted 
and need not be a path. Text is assumed to end with EOT. Lines are 1-based."]
type state = {file: ~str, text: ~[char], index: uint, line: int};


#[doc = "Return type of parse functions."]
type status<T: copy> = result<succeeded<T>, failed>;

#[doc = "new_state will be like the input state except that index and
line may advance. Value is an arbitrary value associated with the parse."]
type succeeded<T: copy> = {new_state: state, value: T};

#[doc = "old_state should be identical to the input state. err_state is
where the error happened."]
type failed = {old_state: state, err_state: state, mesg: ~str};


