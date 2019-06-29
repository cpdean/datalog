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


/*
macro_rules! re_capture (
  ($i:expr, $re:expr) => (
    {
      use $crate::lib::std::result::Result::*;
      use $crate::{Err,error::ErrorKind,IResult};

      use $crate::Slice;
      let re = $crate::lib::regex::Regex::new($re).unwrap();
      if let Some(c) = re.captures(&$i) {
        let v:Vec<_> = c.iter().filter(|el| el.is_some()).map(|el| el.unwrap()).map(|m| $i.slice(m.start()..m.end())).collect();
        let offset = {
          let end = v.last().unwrap();
          end.as_ptr() as usize + end.len() - $i.as_ptr() as usize
        };
        Ok(($i.slice(offset..), v))
      } else {
        let res: IResult<_,_> = Err(Err::Error(error_position!($i, ErrorKind::RegexpCapture)));
        res
      }
    }
  )
);
*/




/*
fn find_a(i: &str) -> IResult<&str, &str> {
    let r = Regex::new("(a)").unwrap();
    match r.captures(&i) {
        Some(caps) => {
            Ok((&i[0..1], &i[0..1]))
        }
        None => Err(ErrorKind::RegexpCapture)

    }
}

*/


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


struct Fact {
    name: String,
    vars: Vec<Variable>
}

struct Rule {
    name: String,
    head: Vec<Variable>,
    body: Vec<Fact>,
}

/*
// something(like, this)
fn fact(i: &str) -> IResult<&Fact, &str> {
    tuple(identifier, delimited(char!('('), arg_list, char!(')')))
}
*/

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
