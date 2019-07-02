#![allow(unused_imports,dead_code)]
/*
 * stores the datalog facts and lets you query them
 */
use crate::ast::{
    Fact,
    Rule,
    BodyExpression,
    EqualityConstraint,
    Statement,
    Variable::Free,
    Variable::Fixed,
};

pub struct Engine {
    facts: Vec<Fact>,
    rules: Vec<Rule>,
}

impl Engine {
    // checks if the fact is present in the engine, referencing literal facts and traversing
    // stored rules
    pub fn query(&self, q: &Fact) -> Vec<&Fact> {
        if let Some(i) = self.facts.iter().find(|&e| e == q) {
            return vec![i];
        } else {
            return vec![];
        }
    }

    pub fn push(&mut self, s: Statement) -> Result<(), String> {
        match s {
            Statement::Fact(f) => self.facts.push(f),
            Statement::Rule(r) => self.rules.push(r),
            Statement::Query(q) => {
                return Err(format!("You cannot save queries to the engine: {:?}", q));
            }
        }
        Ok(())
    }
}

#[test]
fn single_check() {
    let mut e = Engine{ facts: vec![], rules: vec![] };
    e.push(Statement::Fact(
        Fact{ name: "foo".to_owned(), vars: vec![Fixed("bar".to_owned())]}
    )).unwrap();
    let r = e.query(
        &Fact{ name: "foo".to_owned(), vars: vec![Fixed("bar".to_owned())]}
    );
    assert_eq!(r.len(), 1);
}
