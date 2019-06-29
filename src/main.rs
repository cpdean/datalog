#![allow(unused_imports,dead_code)]

#[macro_use]
extern crate nom;

use nom::{IResult,Err};
use nom::error::ErrorKind;
use nom::multi::separated_list;
use nom::bytes::{streaming,complete};
use nom::branch::alt;
use nom::sequence;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use regex::Regex;

fn eval<T>(i: T) -> T {
    i
}


// TODO: is there a way to make free_var's type signature only return Variable::Free?
fn free_var(i: &str) -> IResult<&str, Variable> {
    let re = Regex::new(r"^[A-Z]+\w*").unwrap();
    match re.find(i) {
        Some(m) => {
            let (s, e) = (m.start(), m.end());
            Ok((&i[e..], Variable::Free(i[s..e].to_owned())))
        },
        None => {
            let res: IResult<_,_> = Err(Err::Error(error_position!(i, ErrorKind::RegexpCapture)));
            res
        }
    }
}

fn identifier(i: &str) -> IResult<&str, Variable> {
    let re = Regex::new(r"^[a-z]+\w*").unwrap();
    match re.find(i) {
        Some(m) => {
            let (s, e) = (m.start(), m.end());
            Ok((&i[e..], Variable::Fixed(i[s..e].to_owned())))
        },
        None => {
            let res: IResult<_,_> = Err(Err::Error(error_position!(i, ErrorKind::RegexpCapture)));
            res
        }
    }
}

fn arg_list(i: &str) -> IResult<&str, Vec<Variable>> {
    let white_identifier = sequence::preceded(
        nom::character::complete::multispace0,
        sequence::terminated(
            identifier,
            nom::character::complete::multispace0,
        )
    );

    let white_free_var = sequence::preceded(
        nom::character::complete::multispace0,
        sequence::terminated(free_var,
            nom::character::complete::multispace0,
        )
    );

    separated_list(
        complete::tag(","),
        alt((white_identifier, white_free_var))
    )(i)
}

#[derive(Debug, PartialEq)]
enum Variable {
    Fixed(String),
    Free(String),
}


#[derive(Debug, PartialEq)]
struct Fact {
    name: String,
    vars: Vec<Variable>
}

#[derive(Debug, PartialEq)]
struct Rule {
    name: String,
    head: Vec<Variable>,
    body: Vec<Fact>,
}

// something(like, this)
fn fact(i: &str) -> IResult<&str, Fact> {
    match sequence::tuple((
        identifier,
        sequence::delimited(complete::tag("("), arg_list, complete::tag(")"))
    ))(i) {
        Ok((rest, (ident, args))) => {
            // identifier will only ever be a "fixed" var
            // i need to come up with a cleaner way to do this part
            // maybe this is a red flag that i should not parse the 'business' val directly from
            // the identifier parser?
            let ident_str: String = match ident {
                Variable::Fixed(s) => s,
                // TODO: return an error not a panic on facts that have 'free' style names
                Variable::Free(s) => panic!("{} parsed to 'free' var?", s),
            };
            Ok((rest, Fact{ name: ident_str, vars: args }))
        },
        Err(e) => Err(e)
    }
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

#[test]
fn test_free_var(){
    use Variable::{Free, Fixed};
    assert_eq!(Ok(("", Free("Za".to_owned()))), free_var("Za"));
    assert_eq!(Ok((" goat", Free("Za".to_owned()))), free_var("Za goat"));
    assert_eq!(Ok((" goat", Free("YUS".to_owned()))), free_var("YUS goat"));
    assert_eq!(Err(Err::Error(("yus goat", ErrorKind::RegexpCapture))), free_var("yus goat"));
}

#[test]
fn test_identifier(){
    use Variable::{Free, Fixed};
    assert_eq!(Ok(("", Fixed("za".to_owned()))), identifier("za"));
    assert_eq!(Ok((" goat", Fixed("za".to_owned()))), identifier("za goat"));
    assert_eq!(Err(Err::Error(("YUS goat", ErrorKind::RegexpCapture))), identifier("YUS goat"));
}

#[test]
fn test_arg_list(){
    use Variable::{Free, Fixed};
    assert_eq!(Ok(("", vec![Fixed("za".to_owned())])), arg_list("za"));
    assert_eq!(Ok(("", vec![Fixed("za".to_owned()), Fixed("gg".to_owned())])), arg_list("za,gg"));
    assert_eq!(Ok(("", vec![Fixed("za".to_owned()), Fixed("gg".to_owned())])), arg_list("za, gg"));
    assert_eq!(Ok(("", vec![Fixed("za".to_owned()), Free("Gg".to_owned())])), arg_list("za, Gg"));
}

#[test]
fn test_facts(){
    use Variable::{Free, Fixed};
    assert_eq!(Ok(("", Fact{ name:"something".to_owned(), vars: vec![Fixed("one".to_owned())]})), fact("something(one)"));
    assert_eq!(Ok(("", Fact{ name:"something".to_owned(), vars: vec![Fixed("one".to_owned()), Fixed("two".to_owned())]})), fact("something(one, two)"));
    assert_eq!(Ok(("", Fact{ name:"something".to_owned(), vars: vec![Fixed("one".to_owned()), Free("Two".to_owned())]})), fact("something(one, Two)"));
    assert_eq!(Err(Err::Error((" something(one)", ErrorKind::RegexpCapture))), fact(" something(one)"));
}


#[test]
fn ugh(){
    let re = Regex::new(r"^[A-Z]+\w*").unwrap();
    assert_eq!(true, re.is_match("Aa"));
    assert_eq!(false, re.is_match(" Aa"));
    assert_eq!(true, re.is_match("Za"));
    assert_eq!(true, re.is_match("ZZa"));
    assert_eq!(true, re.is_match("Zaa"));
    assert_eq!(true, re.is_match("Zaa "));
    assert_eq!(true, re.is_match("Z"));
}
