#[doc = "Various utlity functions."];

//import io;
//import io::writer_util;
//import result::*;
//import types::*;

const EOT: char = '\u0003';

// TODO: don't export these.

fn chars_with_eot(s: str) -> [char]
{
    let mut buf = [], i = 0u;
    let len = str::len(s);
    while i < len
    {
        let {ch, next} = str::char_range_at(s, i);
        assert next > i;
        buf += [ch];
        i = next;
    }
    buf += [EOT];
    ret buf;
}
