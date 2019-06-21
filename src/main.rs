use rustyline::error::ReadlineError;
use rustyline::Editor;

use nom::alpha;
use nom::{complete, delimited, do_parse, named, one_of, tag, ws};

named!(syntactic_keyword, tag!("else"));

named!(lowercase_char<char>, one_of!("abcdefghijklmnopqrstuvwxyz"));
named!(uppercase_char<char>, one_of!("ABCDEFGHIJKLMNOPQRSTUVWXYZ"));

named!(identifier<&str>, do_parse!(lowercase_char, alpha));

// fact := <ident>(<ident>)
//ident, tag!("("), ident, tag!(")")
//named!(fact, ws!(delimited!(ident, tag!("("), ident, tag!(")"))));

fn parse(line: &str) {
    let res = syntactic_keyword(line.as_bytes());
    println!("Parsed {:#?}", res);
}

fn eval(i: String) -> String {
    parse(&i);
    i
}

fn main() {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    /*
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    */
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                //rl.add_history_entry(line.as_str());
                let result = eval(line);
                println!("Line: {}", result);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    //rl.save_history("history.txt").unwrap();
}
