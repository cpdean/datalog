#![allow(unused_imports, dead_code)]
extern crate rusqlite;
extern crate time;

use rusqlite::types::ToSql;
use rusqlite::{params, Connection, Result as SQLResult};
use time::Timespec;

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
    let mut e = Engine {
        facts: vec![],
        rules: vec![],
    };
    e.push(Statement::Fact(Fact {
        name: "foo".to_owned(),
        vars: vec![Fixed("bar".to_owned())],
    }))
    .unwrap();
    let r = e.query(&Fact {
        name: "foo".to_owned(),
        vars: vec![Fixed("bar".to_owned())],
    });
    assert_eq!(r.len(), 1);
}

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
