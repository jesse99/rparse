//! Types used by all parse functions.
use Result = result::Result;

// TODO: should be able to get rid of all the owned bounds once
// https://github.com/mozilla/rust/issues/2992 is fixed

/// Type for parse functions.
type Parser<T: Copy Owned> = fn@ (State) -> Status<T>;

/// Input argument for parse functions. File is not interpreted and need 
/// not be a path. Text is assumed to end with EOT. Lines are 1-based.
type State = {file: @~str, text: @[char], index: uint, line: int};

/// Return type of parse functions.
type Status<T: Copy Owned> = Result<Succeeded<T>, Failed>;

/// new_state will be like the input state except that index and line may 
/// advance. Value is an arbitrary value associated with the parse.
type Succeeded<T: Copy Owned> = {new_state: State, value: T};

/// old_state should be identical to the input state. err_state is where 
/// the error happened.
type Failed = {old_state: State, err_state: State, mesg: @~str};

priv type Blah = int;