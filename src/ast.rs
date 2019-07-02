#![allow(unused_imports,dead_code)]

#[derive(Clone, Debug, PartialEq)]
pub enum Variable {
    Fixed(String),
    Free(String),
}

// like "x = Foo" in rule predicates
#[derive(Clone, Debug, PartialEq)]
pub struct EqualityConstraint {
    pub equals: bool,
    pub left: Variable,
    pub right: Variable,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BodyExpression {
    Fact(Fact),
    Equals(EqualityConstraint),
}


#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub name: String,
    pub vars: Vec<Variable>
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub head: Fact,
    pub body: Vec<BodyExpression>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Rule(Rule),
    Fact(Fact),
    Query(Fact),
}

