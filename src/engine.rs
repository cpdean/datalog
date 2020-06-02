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

impl RustEngine {
    /// For when the query asks for an exact record
    fn filter_exact_match(&self, query: Fact) -> Vec<Fact> {
        self.facts
            .iter()
            .filter(|e| query == **e)
            .map(|e| e.clone())
            .collect()
    }

    fn get_relation(&self, name: &str, column_count: usize) -> Vec<&Fact> {
        self.facts
            .iter()
            .filter(|r| r.name == name && r.vars.len() == column_count)
            .collect()
    }
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
        if query.vars.iter().all(|v| match v {
            Fixed(_) => true,
            Free(_) => false,
        }) {
            Ok(Some(self.filter_exact_match(query)))
        } else {
            let mut results = vec![];
            for record in self.get_relation(&query.name, query.vars.len()) {
                // TODO: forget about free vars matching lol. just pretend each free var is a wildcard
                // pairwise comparison of each side
                let mut record_matches = true;
                for (q, r) in query.vars.iter().zip(&record.vars) {
                    match q {
                        v @ Fixed(_) => {
                            if v != r {
                                record_matches = false;
                            }
                        }
                        v @ &Fixed(_) => {
                            if v != r {
                                record_matches = false;
                            }
                        }
                        _v @ Free(_) => {
                            // TODO: somehow trace the freevars.  this has to work both within the same relation but also across relations (joins)
                            continue;
                        }
                    }
                }
                if record_matches {
                    results.push(record.clone());
                }
            }
            Ok(Some(results))
        }
    }
}

#[test]
fn single_check() {
    /*
    check if a single fact gets stored
    > foo(bar).
    > foo(bar)?
    foo(bar).
    */
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

#[test]
fn query_with_free_var() {
    /*
    query for a subset of the facts in a database
    > edge(a, b).
    > edge(a, c).
    > edge(b, d).
    > edge(a, X)?
    edge(a, b).
    edge(a, c).
    */
    let mut e = RustEngine {
        facts: vec![],
        rules: vec![],
    };
    e.push_fact(Statement::Fact(Fact {
        name: "edge".to_owned(),
        vars: vec![Fixed("a".to_owned()), Fixed("b".to_owned())],
    }))
    .unwrap();
    e.push_fact(Statement::Fact(Fact {
        name: "edge".to_owned(),
        vars: vec![Fixed("a".to_owned()), Fixed("c".to_owned())],
    }))
    .unwrap();
    e.push_fact(Statement::Fact(Fact {
        name: "edge".to_owned(),
        vars: vec![Fixed("b".to_owned()), Fixed("d".to_owned())],
    }))
    .unwrap();

    let query = Fact {
        name: "edge".to_owned(),
        vars: vec![Fixed("a".to_owned()), Free("X".to_owned())],
    };

    let r = e.query(query).unwrap().unwrap();
    assert_eq!(r.len(), 2);
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
