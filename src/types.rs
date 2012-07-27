//! The types used by all parse functions.
import result = result::result;

// TODO: should be able to get rid of all the owned bounds once
// https://github.com/mozilla/rust/issues/2992 is fixed

// TODO: we should not be using ~str here because stuff like state
// is treated as a value type and copied. However as of July 2012
// rust still does a very poor job supporting @str.

/// Type for parse functions.
type parser<T: copy owned> = fn@ (state) -> status<T>;

/// Input argument for parse functions. File is not interpreted and need 
/// not be a path. Text is assumed to end with EOT. Lines are 1-based.
type state = {file: ~str, text: @[char], index: uint, line: int};

/// Return type of parse functions.
type status<T: copy owned> = result<succeeded<T>, failed>;

/// new_state will be like the input state except that index and line may 
/// advance. Value is an arbitrary value associated with the parse.
type succeeded<T: copy owned> = {new_state: state, value: T};

/// old_state should be identical to the input state. err_state is where 
/// the error happened.
type failed = {old_state: state, err_state: state, mesg: ~str};