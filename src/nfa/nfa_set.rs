use crate::language::{Label, Language, LanguageError, Match};

use super::{nfa::Transition, state::State, NFA};

/// Build an NFA from multiple NFAs.
/// Allows for detection of multiple matches from a single test.
///
/// The constructed NFA returns the label for the NFA whenever a match is detected.
#[derive(Debug)]
pub struct NFASet(pub NFA);

impl NFASet {
    pub fn build<L>(mut nfas: Vec<(L, NFA)>) -> Result<Self, String>
    where
        L: Into<Label>,
    {
        let mut nfa = if let Some((marker, mut nfa)) = nfas.pop() {
            nfa.new_group_state(marker.into());
            nfa
        } else {
            return Err("At least one nfa must be provided".to_string());
        };

        for (marker, mut next_nfa) in nfas {
            // Offset each state since we append this nfa to the other.
            let add_state = nfa.transitions.len();
            next_nfa.new_group_state(marker.into());

            for state in &mut next_nfa.transitions {
                match state {
                    Transition::Label(_, State(e)) => {
                        if *e == next_nfa.accept.0 {
                            *e = nfa.accept.0;
                        } else {
                            *e += add_state;
                        }
                    }
                    Transition::Split(e1, e2) => {
                        if let Some(State(e1)) = e1 {
                            if *e1 == next_nfa.accept.0 {
                                *e1 = nfa.accept.0;
                            } else {
                                *e1 += add_state;
                            }
                        }
                        if let Some(State(e2)) = e2 {
                            if *e2 == next_nfa.accept.0 {
                                *e2 = nfa.accept.0;
                            } else {
                                *e2 += add_state;
                            }
                        }
                    }
                    Transition::Group(_, State(e)) => {
                        *e += add_state;
                    }
                    Transition::Accept | Transition::Eof => {}
                }
            }

            nfa.transitions.append(&mut next_nfa.transitions);
            let start =
                nfa.new_split_state(Some(nfa.start), Some(State(next_nfa.start.0 + add_state)));
            nfa.start = start;
        }

        Ok(Self(nfa))
    }
}

impl Language for NFASet {
    fn is_match(&self, input: &str) -> Vec<Match> {
        self.0.is_match(input)
    }

    fn to_language(&self) -> String {
        self.0.to_language()
    }

    fn from_language<S: AsRef<str>>(source: S) -> Result<Self, LanguageError> {
        Ok(Self(NFA::from_language(source)?))
    }
}

impl std::fmt::Display for NFASet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        language::{Language, Match},
        nfa::NFA,
    };

    use super::NFASet;

    #[test]

    fn nfa_set() {
        let nfa = NFASet::build(vec![
            ("(a-z)+", NFA::from_language("(a-z)+").unwrap()),
            ("(A-Z)+", NFA::from_language("(A-Z)+").unwrap()),
            ("(0-9)+", NFA::from_language("(0-9)+").unwrap()),
            ("do", NFA::from_language("do").unwrap()),
            ("w|if|b", NFA::from_language("while|if|break").unwrap()),
        ])
        .unwrap();

        assert!(!nfa.is_match("abcdefghijklmnopqrstuvwxyz").is_empty());
        assert!(!nfa.is_match("ABCDEFGHIJKLMNOPQRSTUVWXYZ").is_empty());
        assert!(!nfa.is_match("012931230912312912212").is_empty());
        assert!(!nfa.is_match("do").is_empty());
        assert!(!nfa.is_match("while").is_empty());
        assert!(!nfa.is_match("if").is_empty());
        assert!(!nfa.is_match("break").is_empty());

        let mut matches = nfa.is_match("ifbreak");
        matches.sort_by_key(|m| m.match_size());
        assert_eq!(
            matches,
            vec![
                Match::Group("w|if|b".into(), 2),
                Match::Group("(a-z)+".into(), 7)
            ]
        );

        assert!(nfa.is_match("").is_empty());
        assert!(nfa.is_match("!hello").is_empty());
    }
}
