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
    BodyExpression, EqualityConstraint, Fact, Rule, Statement, Variable, Variable::Fixed,
    Variable::Free,
};

trait DatalogEngine {
    fn push_fact(&mut self, fact: Fact) -> Result<(), String>;
    fn push_rule(&mut self, rule: Rule) -> Result<(), String>;
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

    // pulls records of a relation, leaving filtering on fixed vars
    fn select(&self, query: Fact) -> Vec<Fact> {
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
        results
    }

    fn select_from_rule(&self, query: Fact) -> Vec<Fact> {
        let matching_rules = self.rules.iter().filter(|r| r.head.name == query.name && r.head.vars.len() == query.vars.len()).collect::<Vec<_>>();
        // TODO: hardcode bodies to be only 1 fact for now
        // bar(X) :- foo(X).
        let r = matching_rules[0];
        let r_fact = match &r.body[0] {
            BodyExpression::Fact(f) => f,
            BodyExpression::Equals(_) => panic!("no idea how to deal with eq body rules"),
        };
        self.select(r_fact.clone())
    }
}

impl DatalogEngine for RustEngine {
    // TODO: add constraint to make sure a rule and a fact cannot have the same name
    fn push_fact(&mut self, fact: Fact) -> Result<(), String> {
        self.facts.push(fact);
        Ok(())
    }

    fn push_rule(&mut self, rule: Rule) -> Result<(), String> {
        self.rules.push(rule);
        Ok(())
    }

    fn query(&self, query: Fact) -> Result<Option<Vec<Fact>>, String> {
        dbg!(&self.rules);
        if self.rules.iter().any(|r| r.head.name == query.name) {
            // run the query using the rule's body, which may select from other
            // relations, compute a join, or recursively call itself or other
            // rules
            Ok(Some(self.select_from_rule(query)))
        } else if self.facts.iter().any(|f| f.name == query.name) {
            // the query matches a concrete relation
            // TODO: does datalog's spec allow you to add a fact and then also add a rule so that the rule
            // yields both that single result plus whatever the other definition adds?
            if query.vars.iter().all(|v| match v {
                Fixed(_) => true,
                Free(_) => false,
            }) {
                Ok(Some(self.filter_exact_match(query)))
            } else {
                let results = self.select(query);
                Ok(Some(results))
            }
        } else {
            println!("halp");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(vs: Vec<&str>) -> Vec<Variable> {
        vs.iter()
            .map(|e| {
                if e.chars().next().unwrap().is_uppercase() {
                    Free(e.to_string())
                } else {
                    Fixed(e.to_string())
                }
            })
            .collect()
    }

    fn rule(head: Fact, facts: Vec<Fact>) -> Rule {
        Rule {
            head: head,
            body: facts.into_iter().map(|e| BodyExpression::Fact(e)).collect(),
        }
    }

    fn fact(name: &str, vars: Vec<&str>) -> Fact {
        Fact {
            name: name.to_string(),
            vars: v(vars),
        }
    }

    fn query(name: &str, vars: Vec<&str>) -> Fact {
        Fact {
            name: name.to_string(),
            vars: v(vars),
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

        e.push_fact(fact("foo", vec!["bar"])).unwrap();
        let q = query("foo", vec!["bar"]);

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
        e.push_fact(fact("edge", vec!["a", "b"])).unwrap();
        e.push_fact(fact("edge", vec!["a", "c"])).unwrap();
        e.push_fact(fact("edge", vec!["b", "d"])).unwrap();

        let q = query("edge", vec!["a", "X"]);

        let r = e.query(q).unwrap().unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn query_with_free_var_2() {
        /*
        same as above but just changing things around to see if it still works
        > edge(a, b).
        > edge(a, c).
        > edge(b, d).
        > edge(c, d).
        > edge(j, d).
        > edge(X, d)?
        edge(b, d).
        edge(c, d).
        edge(j, d).
        */
        let mut e = RustEngine {
            facts: vec![],
            rules: vec![],
        };

        e.push_fact(fact("edge", vec!["a", "b"])).unwrap();
        e.push_fact(fact("edge", vec!["a", "c"])).unwrap();
        e.push_fact(fact("edge", vec!["b", "d"])).unwrap();
        e.push_fact(fact("edge", vec!["c", "d"])).unwrap();
        e.push_fact(fact("edge", vec!["j", "d"])).unwrap();

        let q = query("edge", vec!["X", "d"]);

        let r = e.query(q).unwrap().unwrap();
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_rule_projects_new_relation() {
        /*
        A simple rule can project relations backed by concrete facts into a new relation of inferred facts
        > % first add some facts
        > foo(a).
        > foo(b).
        > foo(c).
        > % make a rule to project all 'foo' into a 'bar' relation
        > bar(X) :- foo(X).
        > % query for all bar
        > bar(Q)?
        bar(a).
        bar(b).
        bar(c).
        */

        let mut e = RustEngine {
            facts: vec![],
            rules: vec![],
        };
        e.push_fact(fact("foo", vec!["a"])).unwrap();
        e.push_fact(fact("foo", vec!["b"])).unwrap();
        e.push_fact(fact("foo", vec!["c"])).unwrap();
        e.push_rule(rule(fact("bar", vec!["X"]), vec![fact("foo", vec!["X"])]))
            .unwrap();

        let q = query("bar", vec!["X"]);

        let r = e.query(q).unwrap().unwrap();
        assert_eq!(r.len(), 3);
    }
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
