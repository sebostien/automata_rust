use std::collections::{HashMap, HashSet};

use crate::{
    language::{Language, LanguageError, Match},
    nfa::{State, NFA},
};

pub struct DFA {
    pub alphabet: Vec<char>,
    pub transitions: Vec<HashMap<char, State>>,
    pub start: State,
    pub accept: HashSet<State>,
}

impl From<NFA> for DFA {
    fn from(value: NFA) -> Self {
        todo!()
    }
}

impl Language for DFA {
    fn is_match(&self, input: &str) -> Vec<Match> {
        let mut current = self.start;
        for c in input.chars() {
            match self.transitions[current].get(&c) {
                Some(next) => current = *next,
                None => panic!("Transition table does not contain char: {c}"),
            }
        }

        if self.accept.contains(&current) {
            vec![Match::NoGroup(input.len())]
        } else {
            vec![]
        }
    }

    fn to_language(&self) -> String {
        todo!()
    }

    fn try_from_language<S: AsRef<str>>(source: S) -> Result<Self, LanguageError> {
        NFA::try_from_language(source).map(DFA::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::graph_display::DiGraph;

    use super::*;

    #[test]
    fn dfa() {
        let dfa = DFA {
            alphabet: vec!['0', '1'],
            transitions: vec![
                HashMap::from([('0', State(1)), ('1', State(0))]),
                HashMap::from([('0', State(0)), ('1', State(1))]),
            ],
            start: State(0),
            accept: HashSet::from([State(0)]),
        };

        let graph: DiGraph = (&dfa).into();
        std::fs::write("./graph.svg", graph.to_string()).expect("Could not write data to file!");

        println!("{:?}", dfa.is_match("01"));

        assert!(dfa.is_match("01").is_empty());
        assert!(dfa.is_match("0100").is_empty());

        assert!(!dfa.is_match("010").is_empty());
        assert!(!dfa.is_match("00111010").is_empty());
    }
}
