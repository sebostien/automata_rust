use graphviz_rust::attributes::{arrowhead, shape, EdgeAttributes, NodeAttributes};
use graphviz_rust::cmd::{Format, Layout};
use graphviz_rust::dot_generator::{edge, graph, id, node, node_id};
use graphviz_rust::dot_structures::{Edge, EdgeTy, Graph, Id, Node, NodeId, Vertex};
use graphviz_rust::exec_dot;
use graphviz_rust::printer::{DotPrinter, PrinterContext};

use crate::dfa::DFA;
use crate::nfa::State;
use crate::nfa::Transition;
use crate::nfa::NFA;

pub struct DiGraph(graphviz_rust::dot_structures::Graph);

impl From<&NFA> for DiGraph {
    fn from(nfa: &NFA) -> Self {
        let mut nodes = vec![];
        let mut edges = vec![];

        for (state, transition) in nfa.transitions.iter().enumerate() {
            let state = State(state);
            if state == nfa.accept || state == nfa.eof {
                nodes.push(node!(state; NodeAttributes::shape(shape::doublecircle)));
            } else if state == nfa.start {
                nodes.push(node!(state));
                nodes.push(node!("start"; NodeAttributes::shape(shape::none)));
                edges.push(edge!(node_id!("start") => node_id!(state); 
                                 EdgeAttributes::arrowhead(arrowhead::normal)));
            } else {
                nodes.push(node!(state));
            }

            match transition {
                Transition::Label(l, e) => {
                    edges.push(edge!(node_id!(state) => node_id!(e);
                            EdgeAttributes::arrowhead(arrowhead::normal),
                            EdgeAttributes::label(format!("\"'\\{l}'\""))
                    ));
                }
                &Transition::Split(e1, e2) => {
                    if let Some(e1) = e1 {
                        edges.push(edge!(node_id!(state) => node_id!(e1)));
                    }
                    if let Some(e2) = e2 {
                        edges.push(edge!(node_id!(state) => node_id!(e2)));
                    }
                }
                Transition::Accept => {}
                Transition::Group(g, e) => {
                    edges.push(edge!(node_id!(state) => node_id!(e);
                                EdgeAttributes::arrowhead(arrowhead::normal),
                                EdgeAttributes::label(format!("\"G: {g}\""))));
                }
                Transition::Eof => {}
            }
        }

        let mut graph: graphviz_rust::dot_structures::Graph = graph!(strict di id!("G"));
        for node in nodes {
            graph.add_stmt(node.into());
        }

        for edge in edges {
            graph.add_stmt(edge.into());
        }

        Self(graph)
    }
}

impl From<&DFA> for DiGraph {
    fn from(dfa: &DFA) -> Self {
        let mut nodes = vec![];
        let mut edges = vec![];

        for (state, transitions) in dfa.transitions.iter().enumerate() {
            let state = State(state);
            if dfa.accept.contains(&state) {
                nodes.push(node!(state; NodeAttributes::shape(shape::doublecircle)));
            } else {
                nodes.push(node!(state));
            }

            if state == dfa.start {
                nodes.push(node!("start"; NodeAttributes::shape(shape::none)));
                edges.push(edge!(node_id!("start") => node_id!(state); 
                                 EdgeAttributes::arrowhead(arrowhead::normal)));
            }

            for (c, e) in transitions {
                edges.push(edge!(node_id!(state) => node_id!(e);
                        EdgeAttributes::arrowhead(arrowhead::normal),
                        EdgeAttributes::label(format!("\"{c}\""))
                ));
            }
        }

        let mut graph: graphviz_rust::dot_structures::Graph = graph!(strict di id!("G"));
        for node in nodes {
            graph.add_stmt(node.into());
        }

        for edge in edges {
            graph.add_stmt(edge.into());
        }

        Self(graph)
    }
}

impl std::fmt::Display for DiGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dot = self.0.print(&mut PrinterContext::default());

        match exec_dot(dot, vec![Format::Svg.into(), Layout::Dot.into()]) {
            Ok(s) => s.fmt(f),
            Err(e) => {
                eprintln!("{}", e);
                Err(std::fmt::Error)
            }
        }
    }
}
