#![allow(unused_imports, dead_code)]
extern crate rusqlite;
extern crate time;

use rusqlite::types::ToSql;
use rusqlite::{params, Connection, Result as SQLResult};
use std::io::Error;
use time::Timespec;

/*
 * stores the datalog facts and lets you query them
 */
use crate::ast::{
    BodyExpression, EqualityConstraint, Fact, Rule, Statement, Variable::Fixed, Variable::Free,
};

trait DatalogEngine {
    fn push_fact(&mut self, fact: Statement) -> Result<(), String>;
    fn push_rule(&mut self, rule: Statement) -> Result<(), String>;
    fn query(&self, query: Fact) -> Result<Option<Vec<Fact>>, String>;
}

/// RustEngine is a datalog engine that implements its internals via loops and stuff in Rust
pub struct RustEngine {
    facts: Vec<Fact>,
    rules: Vec<Rule>,
}

impl DatalogEngine for RustEngine {
    fn push_fact(&mut self, fact: Statement) -> Result<(), String> {
        match fact {
            Statement::Fact(f) => {
                self.facts.push(f);
                Ok(())
            }
            Statement::Rule(_) | Statement::Query(_) => {
                Err("only accepts fact statements".to_string())
            }
        }
    }

    fn push_rule(&mut self, rule: Statement) -> Result<(), String> {
        match rule {
            Statement::Rule(r) => {
                self.rules.push(r);
                Ok(())
            }
            Statement::Fact(_) | Statement::Query(_) => {
                Err("only accepts rule statements".to_string())
            }
        }
    }

    fn query(&self, query: Fact) -> Result<Option<Vec<Fact>>, String> {
        let generated_facts = self.facts.clone();
        let results = generated_facts
            .iter()
            .filter(|e| query == **e)
            .map(|e| e.clone())
            .collect();
        Ok(Some(results))
    }
}

#[test]
fn single_check() {
    let mut e = RustEngine {
        facts: vec![],
        rules: vec![],
    };
    e.push_fact(Statement::Fact(Fact {
        name: "foo".to_owned(),
        vars: vec![Fixed("bar".to_owned())],
    }))
    .unwrap();
    let q = Fact {
        name: "foo".to_owned(),
        vars: vec![Fixed("bar".to_owned())],
    };

    let r = e.query(q).unwrap().unwrap();
    assert_eq!(r.len(), 1);
}

// TODO: these are just some tests to play around with rusqlite
#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    time_created: Timespec,
    data: Option<Vec<u8>>,
}

#[test]
fn test_things() -> SQLResult<()> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE person (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL,
                  time_created    TEXT NOT NULL,
                  data            BLOB
                  )",
        params![],
    )?;
    let me = Person {
        id: 0,
        name: "Steven".to_string(),
        time_created: time::get_time(),
        data: None,
    };
    conn.execute(
        "INSERT INTO person (name, time_created, data)
                  VALUES (?1, ?2, ?3)",
        params![me.name, me.time_created, me.data],
    )?;

    let mut stmt = conn.prepare("SELECT id, name, time_created, data FROM person")?;
    let person_iter = stmt.query_map(params![], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            time_created: row.get(2)?,
            data: row.get(3)?,
        })
    })?;

    let names: Vec<String> = person_iter.map(|p| p.unwrap().name).collect();

    assert_eq!(vec!["Steven"], names);

    Ok(())
}
