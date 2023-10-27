//! Implementation of [Thompson's Construction]
//!
//! [Thompson's Construction]: <https://en.wikipedia.org/wiki/Thompson%27s_construction>
//!
//! Resources:
//!
//! <https://swtch.com/~rsc/regexp/regexp1.html>
//!

#![allow(soft_unstable)]

use std::collections::{HashMap, HashSet};

use super::state::State;

use crate::{
    language::{CompileError, Label, Language, LanguageError, Match},
    parse::{Lit, Postfix, Token},
    table::Table,
};

impl<T> std::ops::Index<State> for Vec<T> {
    type Output = T;

    fn index(&self, index: State) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> std::ops::IndexMut<State> for Vec<T> {
    fn index_mut(&mut self, index: State) -> &mut Self::Output {
        &mut self[index.0]
    }
}

#[derive(Debug, Clone)]
pub enum Transition {
    Label(Lit, State),
    Split(Option<State>, Option<State>),
    Group(Label, State),
    Eof,
    Accept,
}

#[derive(Debug)]
pub struct NFA {
    /// Each state has it's own row of transitions.
    /// Thus `transitions.len() == num_states`
    pub transitions: Vec<Transition>,
    pub start: State,
    /// Only a single accepting state.
    pub accept: State,
    /// State that don't accept any more tokens.
    pub eof: State,
}

impl NFA {
    #[must_use]
    pub fn new() -> Self {
        let nfa = Self {
            transitions: vec![Transition::Eof],
            eof: State(0),
            // Is changed when regex is compiled
            accept: State(0),
            // Is changed when regex is compiled
            start: State(0),
        };
        nfa
    }
}

impl std::ops::Index<State> for NFA {
    type Output = Transition;

    fn index(&self, index: State) -> &Self::Output {
        &self.transitions[index]
    }
}

impl std::ops::IndexMut<State> for NFA {
    fn index_mut(&mut self, index: State) -> &mut Self::Output {
        &mut self.transitions[index]
    }
}

impl NFA {
    #[must_use]
    pub(crate) fn new_label_state(&mut self, label: Lit) -> State {
        let state = State(self.transitions.len());
        self.transitions.push(Transition::Label(label, state));
        state
    }

    #[must_use]
    pub(crate) fn new_split_state(&mut self, e1: Option<State>, e2: Option<State>) -> State {
        self.transitions.push(Transition::Split(e1, e2));
        State(self.transitions.len() - 1)
    }

    #[must_use]
    pub(crate) fn new_accept_state(&mut self) -> State {
        self.transitions.push(Transition::Accept);
        State(self.transitions.len() - 1)
    }

    /// Insert a new group state at the start of the NFA.
    pub(crate) fn new_group_state(&mut self, marker: Label) {
        self.transitions.push(Transition::Group(marker, self.start));
        self.start = State(self.transitions.len() - 1);
    }

    fn patch(&mut self, from: &Frag, to: State) {
        for outp in &from.out {
            match &mut self[*outp] {
                Transition::Label(_, e) => *e = to,
                Transition::Split(_, e2) => {
                    *e2 = Some(to);
                }
                Transition::Group(_, _) => panic!(),
                Transition::Accept => panic!(),
                Transition::Eof => panic!(),
            }
        }
    }
}

#[derive(Debug)]
struct Frag {
    start: State,
    out: Vec<State>,
}

impl NFA {
    /// Compile postfix notation into an NFA.
    ///
    /// # Errors
    ///
    /// Fails if the postfix stack contians '(' or ')' tokens or has invalid syntax.
    pub fn compile(postfix: Postfix) -> Result<Self, CompileError> {
        let mut nfa = Self::new();

        nfa.accept = nfa.new_accept_state();

        let mut stack: Vec<Frag> = vec![];

        for tok in postfix.tokens {
            match tok {
                Token::KleeneS => {
                    //   -> e
                    //  /    \
                    // s <----
                    //  \
                    //   -------->
                    let e = stack.pop().ok_or(CompileError::EmptyStack {
                        token: Token::KleeneS,
                    })?;
                    let s = nfa.new_split_state(Some(e.start), None);
                    nfa.patch(&e, s);
                    let e = Frag {
                        start: s,
                        out: vec![s],
                    };
                    stack.push(e);
                }
                Token::Union => {
                    //  /-> e1 ->
                    // s
                    //  \-> e2 ->
                    let mut e2 = stack.pop().unwrap();
                    let mut e1 = stack.pop().unwrap();
                    let s = nfa.new_split_state(Some(e1.start), Some(e2.start));
                    e1.out.append(&mut e2.out);
                    e1.start = s;
                    stack.push(e1);
                }
                Token::Concat => {
                    // e1 -> e2 ->
                    let e2 = stack.pop().unwrap();
                    let e1 = stack.pop().unwrap();
                    nfa.patch(&e1, e2.start);

                    stack.push(Frag {
                        start: e1.start,
                        out: e2.out,
                    });
                }
                Token::KleeneP => {
                    //  -----
                    // /    |
                    // v    |
                    // e -> s ->
                    let e = stack.pop().unwrap();
                    let s = nfa.new_split_state(Some(e.start), None);
                    nfa.patch(&e, s);
                    let e = Frag {
                        start: e.start,
                        out: vec![s],
                    };
                    stack.push(e);
                }
                Token::Optional => {
                    //   -> e --\
                    //  /        v
                    // s
                    //  \        ^
                    //   -------/
                    let mut e = stack.pop().unwrap();
                    let s = nfa.new_split_state(Some(e.start), None);
                    e.out.push(s);
                    e.start = s;
                    stack.push(e);
                }
                Token::Range => {
                    return Err(CompileError::UnexpectedRange);
                }
                Token::OParen => {
                    return Err(CompileError::UnexpectedOpenParen);
                }
                Token::CParen => {
                    return Err(CompileError::UnexpectedCloseParen);
                }
                Token::Eof => {
                    //   eof
                    // s -> accept
                    let s = nfa.new_split_state(Some(nfa.eof), None);
                    stack.push(Frag {
                        start: s,
                        out: vec![],
                    });
                }
                Token::Lit(c) => {
                    //   c
                    // s ->
                    let s = nfa.new_label_state(c);
                    stack.push(Frag {
                        start: s,
                        out: vec![s],
                    });
                }
            }
        }

        if let (1, Some(e)) = (stack.len(), stack.pop()) {
            nfa.start = e.start;
            nfa.patch(&e, nfa.accept);
            Ok(nfa)
        } else {
            Err(CompileError::NonUnaryStack { size: stack.len() })
        }
    }
}

impl NFA {
    #[must_use]
    pub fn generate<const MAX_LEN: usize>(&self) -> Vec<String> {
        let mut done = HashSet::new();
        let mut states = vec![(String::new(), self.start)];

        while let Some((mut s, state)) = states.pop() {
            if s.len() > MAX_LEN {
                continue;
            }

            match &self[state] {
                Transition::Label(l, e) => {
                    match l {
                        Lit::Any => todo!(),
                        Lit::Char(c) => s.push(*c),
                        Lit::Range(c) => s.push(*c.start()),
                    }
                    states.push((s, *e));
                }
                &Transition::Split(e1, e2) => {
                    if let Some(e1) = e1 {
                        states.push((s.clone(), e1));
                    }

                    if let Some(e2) = e2 {
                        states.push((s.clone(), e2));
                    }
                }
                Transition::Accept => {
                    done.insert(s);
                }
                &Transition::Group(_, e) => {
                    states.push((s.clone(), e));
                }
                Transition::Eof => {
                    done.insert(s);
                }
            }
        }

        done.into_iter().collect()
    }

    /// Returns true if `self` can only match a single fixed string.
    pub fn is_fixed(&self) -> bool {
        let mut states = vec![self.start];

        while let Some(state) = states.pop() {
            match &self[state] {
                Transition::Label(l, e) => {
                    if !matches!(l, Lit::Char(_)) {
                        return false;
                    }
                    states.push(*e);
                }
                &Transition::Split(e1, e2) => {
                    if e1.is_some() | e2.is_some() {
                        return false;
                    }
                }
                _ => {}
            }
        }

        true
    }
}

#[derive(Debug)]
struct Step {
    /// The current char in the input string.
    current_char: char,
    /// Number of bytes of the input consumed thus far.
    consumed: usize,
    /// Contains a number for each state.
    /// If `step_list[state] == step` then we have reached the state already.
    step_list: Vec<usize>,
    /// The current step.
    step: usize,
}

impl Step {
    #[must_use]
    fn new(num_states: usize) -> Self {
        Self {
            current_char: 0 as char,
            consumed: 0,
            step_list: (0..num_states).into_iter().map(|_| 0).collect(),
            step: 1,
        }
    }

    #[must_use]
    fn is_visited(&self, state: State) -> bool {
        self.step_list[state] == self.step
    }

    fn set_visited(&mut self, state: State) {
        self.step_list[state] = self.step;
    }

    fn next_step(&mut self, current_char: char) {
        self.step += 1;
        self.current_char = current_char;
        // The char might be more than one byte.
        self.consumed += current_char.len_utf8();
    }
}

impl NFA {
    fn add_state(
        &self,
        step: &mut Step,
        list: &mut Vec<(Option<Label>, State)>,
        matches: &mut HashMap<Option<Label>, usize>,
        group: Option<Label>,
        state: State,
    ) {
        if step.is_visited(state) {
            return;
        };

        match &self[state] {
            &Transition::Split(e1, e2) => {
                if let Some(e1) = e1 {
                    self.add_state(step, list, matches, group, e1);
                }
                if let Some(e2) = e2 {
                    self.add_state(step, list, matches, group, e2);
                }
            }
            Transition::Group(l, e) => self.add_state(step, list, matches, Some(l.clone()), *e),
            Transition::Label(_, _) | Transition::Accept => {
                step.set_visited(state);
                list.push((group.clone(), state));

                if state == self.accept {
                    matches.insert(group.clone(), step.consumed);
                }
            }
            Transition::Eof => {
                step.set_visited(state);
                list.push((group, state));
            }
        }
    }

    /// Step each state in `current_list` with `c`, following any eps-closuers.
    /// Returns `true` if the accepting state has been reached.
    fn step(
        &self,
        step: &mut Step,
        current_list: &Vec<(Option<Label>, State)>,
        next_list: &mut Vec<(Option<Label>, State)>,
        matches: &mut HashMap<Option<Label>, usize>,
    ) {
        debug_assert!(next_list.is_empty());

        for (group, state) in current_list {
            match &self[*state] {
                Transition::Label(cond, e) => {
                    if cond.accepts(step.current_char) {
                        self.add_state(step, next_list, matches, *group, *e);
                    }
                }
                Transition::Split(_, _) | Transition::Group(_, _) => unreachable!(),
                Transition::Accept | Transition::Eof => {
                    // The accept state is already in matches
                    // We reject the eof state by simply not adding this state to the next iteration
                }
            }
        }
    }
}

impl From<(Option<Label>, usize)> for Match {
    fn from((ol, size): (Option<Label>, usize)) -> Self {
        match ol {
            Some(l) => Self::Group(l, size),
            None => Self::NoGroup(size),
        }
    }
}

impl Language for NFA {
    fn is_match(&self, input: &str) -> Vec<Match> {
        let mut current_list = Vec::with_capacity(self.transitions.len());
        let mut next_list = Vec::with_capacity(self.transitions.len());

        let mut matches = HashMap::new();

        let mut step = Step::new(self.transitions.len());

        // Follow any eps-closuers at the start
        self.add_state(&mut step, &mut current_list, &mut matches, None, self.start);

        for c in input.chars() {
            step.next_step(c);

            self.step(&mut step, &current_list, &mut next_list, &mut matches);

            std::mem::swap(&mut current_list, &mut next_list);
            next_list.truncate(0);
        }

        // Add any Eof states still on the stack
        let current_list = current_list
            .into_iter()
            .filter_map(|(group, state)| (state == self.eof).then_some((group, input.len())));

        matches
            .into_iter()
            .chain(current_list)
            .map(|(l, s)| (l, s).into())
            .collect()
    }

    fn to_language(&self) -> String {
        todo!()
    }

    fn try_from_language<S: AsRef<str>>(source: S) -> Result<Self, LanguageError> {
        let postfix = source.as_ref().parse().map_err(LanguageError::ParseError)?;
        Self::compile(postfix).map_err(LanguageError::CompileError)
    }
}

impl std::fmt::Display for NFA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers = ["Type", "State", "Label", "e1", "e2"].map(String::from);

        let mut data = vec![];

        for (state, transition) in self.transitions.iter().enumerate() {
            let mut ty = if State(state) == self.start {
                "Start:"
            } else if State(state) == self.accept {
                "Accept:"
            } else if State(state) == self.eof {
                "Eof"
            } else {
                ""
            }
            .to_string();

            let mut lab = String::new();
            let mut edge1 = String::new();
            let mut edge2 = String::new();

            match transition {
                Transition::Label(label, e) => {
                    lab = label.to_string();
                    edge1 = e.to_string();
                }
                Transition::Split(e1, e2) => {
                    edge1 = e1.map(|e1| e1.to_string()).unwrap_or(String::new());
                    edge2 = e2.map(|e2| e2.to_string()).unwrap_or(String::new());
                }
                Transition::Group(g, e) => {
                    ty = "G:".to_string();
                    lab = g.to_string();
                    edge1 = e.to_string();
                }
                Transition::Accept | Transition::Eof => {
                    // Covered in `ty` above
                }
            }

            data.push([ty, state.to_string(), lab, edge1, edge2]);
        }

        let table = Table::<5>::new(headers, data);
        table.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_gen<const MAX_LEN: usize>(pattern: &str, possible: usize) {
        let nfa = NFA::try_from_language(pattern).unwrap();
        let gen = nfa.generate::<MAX_LEN>();
        if gen.len() != possible {
            std::fs::write("./gen.txt", gen.join("\n")).expect("Failed to write");
            panic!(
                "\nExpected {possible} alternatives but found {}.\n{nfa}\nAlternatives was written to ./gen.txt",
                gen.len()
            );
        }
    }

    /// Some languages with a known set of unique words with max length N.
    #[test]
    fn gen() {
        test_gen::<100>("AB|AC|CB|DC", 4);
        test_gen::<100>("A|(A?B)|C", 4);
        test_gen::<100>("(A|B)?", 3);
        test_gen::<100>("A|CB", 2);
        test_gen::<100>("A(A|B)?C((A|B)|(C|D))", 12);
        test_gen::<8>("(A+)(B*)(C?)(D+|E?)", 253);
    }

    #[test]
    fn matches() {
        let nfa: NFA = NFA::try_from_language("A?A?A*B").unwrap();
        assert_eq!(nfa.is_match("BB"), (vec![Match::NoGroup(1)]));
        assert_eq!(nfa.is_match("AB"), (vec![Match::NoGroup(2)]));
        assert_eq!(nfa.is_match("AAB"), (vec![Match::NoGroup(3)]));
        assert_eq!(nfa.is_match("AAAB"), (vec![Match::NoGroup(4)]));
        assert_eq!(nfa.is_match("AAAAB"), (vec![Match::NoGroup(5)]));
        assert_eq!(nfa.is_match("BAAAAB"), (vec![Match::NoGroup(1)]));
        assert!(nfa.is_match("AAA").is_empty());
        assert!(nfa.is_match("CAAAAB").is_empty());

        let nfa: NFA = NFA::try_from_language("(A|B)+").unwrap();
        assert!(nfa.is_match("").is_empty());
        assert_eq!(nfa.is_match("AAAA"), vec![Match::NoGroup(4)]);
        assert_eq!(nfa.is_match(&"A".repeat(20)), vec![Match::NoGroup(20)]);
        assert_eq!(nfa.is_match(&"B".repeat(20)), vec![Match::NoGroup(20)]);
        assert_eq!(
            nfa.is_match(&"ABAAB".repeat(20)),
            vec![Match::NoGroup(5 * 20)]
        );
        assert!(nfa.is_match(&"a".repeat(20)).is_empty());

        let nfa: NFA = NFA::try_from_language("(A|B)?C?").unwrap();
        assert_eq!(nfa.is_match(""), (vec![Match::NoGroup(0)]));
        assert_eq!(nfa.is_match("A"), (vec![Match::NoGroup(1)]));
        assert_eq!(nfa.is_match("B"), (vec![Match::NoGroup(1)]));
        assert_eq!(nfa.is_match("C"), (vec![Match::NoGroup(1)]));
        assert_eq!(nfa.is_match("AC"), (vec![Match::NoGroup(2)]));

        let nfa: NFA = NFA::try_from_language(r"\n|\t+").unwrap();
        assert!(nfa.is_match("").is_empty());
        assert_eq!(nfa.is_match("\t\t"), (vec![Match::NoGroup(2)]));
        assert_eq!(nfa.is_match("\n"), (vec![Match::NoGroup(1)]));
        assert_eq!(nfa.is_match("\t\n"), (vec![Match::NoGroup(1)]));
        assert_eq!(nfa.is_match("\n\t"), (vec![Match::NoGroup(1)]));
        assert!(nfa.is_match("\\n\\t").is_empty());
        assert!(nfa.is_match(r"\n\t").is_empty());
    }

    #[test]
    fn eof() {
        let nfa: NFA = NFA::try_from_language("a$").unwrap();
        assert_eq!(nfa.is_match("a"), (vec![Match::NoGroup(1)]));
        assert_eq!(nfa.is_match(""), vec![]);
        assert_eq!(nfa.is_match("aa"), vec![]);

        let nfa: NFA = NFA::try_from_language("a$|b+$").unwrap();
        assert_eq!(nfa.is_match("a"), (vec![Match::NoGroup(1)]));
        assert_eq!(nfa.is_match("b"), vec![Match::NoGroup(1)]);
        assert_eq!(nfa.is_match("bbb"), vec![Match::NoGroup(3)]);
        assert_eq!(nfa.is_match("ab"), vec![]);
        assert_eq!(nfa.is_match("bba"), vec![]);

        let nfa: NFA = NFA::try_from_language("$").unwrap();
        assert_eq!(nfa.is_match(""), vec![Match::NoGroup(0)]);
    }

    extern crate test;
    use test::Bencher;

    /// This is not a good benchmark.
    /// It's a simple test to check whether a particular optimization has any effect.
    ///
    /// Previous iterations:
    /// - 10,510,434 ns/iter (+/- 210,810)
    /// - 10,631,001 ns/iter (+/- 198,757)   After chaning is_match to return `Option<usize>`.
    /// - 18,495,653 ns/iter (+/- 1,023,148) After adding capturing groups and char-classes.
    /// - 11,306,364 ns/iter (+/- 419,921)   Add &mut to matches HashMap, avoids redundant loop.
    #[bench]
    fn bench_matches(b: &mut Bencher) {
        const N: usize = 250;

        let pattern = "A?".repeat(N) + &"A".repeat(N);
        let input = &"A".repeat(N);

        let nfa: NFA = NFA::try_from_language(pattern).unwrap();

        assert!(!nfa.is_match(input).is_empty());

        b.iter(|| !nfa.is_match(input).is_empty());
    }
}
