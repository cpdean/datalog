#![allow(unused_imports,dead_code)]

#[macro_use]
extern crate nom;

use nom::{IResult,Err};
use nom::error::ErrorKind;
use nom::multi::separated_list;
use nom::bytes::{streaming,complete};
use nom::branch::alt;
use nom::sequence;
use nom::combinator::map;

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

#[derive(Clone, Debug, PartialEq)]
enum Variable {
    Fixed(String),
    Free(String),
}

// like "x = Foo" in rule predicates
#[derive(Clone, Debug, PartialEq)]
struct EqualityConstraint {
    equals: bool,
    left: Variable,
    right: Variable,
}

#[derive(Clone, Debug, PartialEq)]
enum BodyExpression {
    Fact(Fact),
    Equals(EqualityConstraint),
}


#[derive(Clone, Debug, PartialEq)]
struct Fact {
    name: String,
    vars: Vec<Variable>
}

#[derive(Clone, Debug, PartialEq)]
struct Rule {
    head: Fact,
    body: Vec<BodyExpression>,
}

#[derive(Clone, Debug, PartialEq)]
enum Statement {
    Rule(Rule),
    Fact(Fact)
}

fn equality_constraint(i: &str) -> IResult<&str, EqualityConstraint> {
    nom::combinator::map(
        sequence::tuple((
            alt((free_var, identifier)),
            sequence::preceded(nom::character::complete::multispace0,
                nom::combinator::map(
                    alt((complete::tag("="), complete::tag("!="))), |e| e == "=")
            ),
            sequence::preceded(nom::character::complete::multispace0,
                alt((free_var, identifier))
            ),
        )),
        |(left, op, right)| EqualityConstraint { left: left, equals: op, right: right }
    )(i)
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

// TODO: I don't like how i'm using fact to both mean a component in a rule but also a fact
// persisted to the datalog engine
fn fact_statement(i: &str) -> IResult<&str, Fact> {
    sequence::terminated(
        sequence::preceded(nom::character::complete::multispace0, fact),
        sequence::preceded(nom::character::complete::multispace0, complete::tag("."))
    )(i)
}

// not going to enforce semantics of free vars yet, validate that later i guess
// for now just trying to parse this structure:
// cousin(X, Y) :- siblings(A, B), parent(A, X), parent(B, Y)
fn rule_statement(i: &str) -> IResult<&str, Rule> {
    let either_predicate_or_equality_constraint =
        sequence::preceded(
            nom::character::complete::multispace0,
            alt((
                map(fact, |f| BodyExpression::Fact(f)),
                map(equality_constraint, |e| BodyExpression::Equals(e))
            ))
        );

    let the_rule = sequence::separated_pair(
        sequence::preceded(nom::character::complete::multispace0, fact),
        sequence::preceded(nom::character::complete::multispace0, complete::tag(":-")),
        sequence::terminated(
            separated_list(
                sequence::preceded(nom::character::complete::multispace0, complete::tag(",")),
                either_predicate_or_equality_constraint
            ),
            sequence::preceded(nom::character::complete::multispace0, complete::tag("."))
        )
    )(i);
    match the_rule {
        Ok((rest, (head, body))) => {
            // identifier will only ever be a "fixed" var
            // i need to come up with a cleaner way to do this part
            // maybe this is a red flag that i should not parse the 'business' val directly from
            // the identifier parser?
            Ok((rest, Rule{ head: head, body: body }))
        },
        Err(e) => Err(e)
    }
}


fn statement(i: &str) -> IResult<&str, Statement> {
    alt((
        nom::combinator::map(rule_statement, |e| Statement::Rule(e)),
        nom::combinator::map(fact_statement, |e| Statement::Fact(e))
    ))(i)
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
fn test_equality_constraint() {
    use Variable::{Free, Fixed};
    fn _free(n: &str) -> Variable {
        Free(n.to_owned())
    }
    fn _fixed(n: &str) -> Variable {
        Fixed(n.to_owned())
    }
    assert_eq!(Ok(("", EqualityConstraint{ left: _fixed("za") , equals: true , right: _fixed("za") } )), equality_constraint("za = za"));
    assert_eq!(Ok(("", EqualityConstraint{ left: _free("Aa") , equals: true , right: _fixed("za") } )), equality_constraint("Aa = za"));
    assert_eq!(Ok(("", EqualityConstraint{ left: _free("Aa") , equals: true , right: _fixed("za") } )), equality_constraint("Aa =  za"));
    assert_eq!(Ok(("", EqualityConstraint{ left: _free("Aa") , equals: false , right: _free("Za") } )), equality_constraint("Aa != Za"));
}

#[test]
fn test_rules(){
    use Variable::{Free, Fixed};
    use BodyExpression::{Equals as BE, Fact as BF};
    fn _free(n: &str) -> Variable {
        Free(n.to_owned())
    }
    fn _fixed(n: &str) -> Variable {
        Fixed(n.to_owned())
    }
    fn _fact(n: &str, a: Vec<Variable>) -> Fact {
        Fact{ name: n.to_owned(), vars: a }
    }
    fn _rule(h: Fact, a: Vec<BodyExpression>) -> Rule {
        Rule { head: h, body: a }
    }
    // simple one-to-one relationship
    assert_eq!(
        Ok(("",
            _rule(
                _fact("good", vec![_fixed("one")]),
                vec![
                    BF(_fact("gut", vec![_fixed("one")]))
               ])
        )), rule_statement("good(one) :- gut(one).")
    );

    // check if other fact related
    assert_eq!(
        Ok(("",
            _rule(
                _fact("good", vec![_fixed("one"), _free("Two")]),
                vec![
                    BF(_fact("gut", vec![_fixed("one")])),
                    BF(_fact("foo", vec![_fixed("one"), _free("Two")]))
               ])
        )), rule_statement("good(one, Two) :- gut(one), foo(one, Two).")
    );

    // add equality check predicate test, for expressions like x = y, x != z.
    assert_eq!(
        Ok(("",
            _rule(
                _fact("good", vec![_fixed("one"), _free("Two")]),
                vec![
                    BF(_fact("gut", vec![_fixed("one")])),
                    BF(_fact("foo", vec![_fixed("one"), _free("Two")])),
                    BE(EqualityConstraint{ left: _fixed("one"), equals: true, right: _free("Two") })
               ])
        )), rule_statement("good(one, Two) :- gut(one), foo(one, Two), one = Two.")
    );

    // ensure formattiing parses to same obj
    let check = _rule(
        _fact("good", vec![_fixed("one"), _free("Two")]),
        vec![
            BF(_fact("gut", vec![_fixed("one")])),
            BF(_fact("foo", vec![_fixed("one"), _free("Two")]))
        ]
    );

    assert_eq!(Ok(("", check.clone())), rule_statement("good(one, Two) :- gut(one), foo(one, Two)."));
    assert_eq!(Ok(("", check.clone())), rule_statement("  good(one, Two) :- gut(one), foo(one, Two)."));
    assert_eq!(Ok(("", check.clone())), rule_statement("  good(  one,  Two)  :- gut(one), foo(one, Two)."));
    assert_eq!(Ok(("", check.clone())), rule_statement("  good(  one,  Two)  :-    gut( one)   , foo(    one, Two )  ."));



    // TODO: thiis is a bad(opaque) error message
    assert_eq!(Err(Err::Error(("", ErrorKind::Tag))), rule_statement(" something(one)"));
}

#[test]
fn test_rule_statement(){
    let (correct, result) = match statement("f(a) :- g(a).") {
        Ok((rest, Statement::Rule(r))) => (true, Ok((rest, Statement::Rule(r)))),
        x => (false, x)
    };
    assert!(correct, "was not rule {:?}", result);
}

#[test]
fn test_fact_statement(){
    let (correct, result) = match statement("f(a).") {
        Ok((rest, Statement::Fact(f))) => (true, Ok((rest, Statement::Fact(f)))),
        x => (false, x)
    };
    assert!(correct, "was not fact {:#?}", result);
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
