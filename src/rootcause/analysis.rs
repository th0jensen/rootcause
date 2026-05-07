use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use serde::Deserialize;

use crate::rootcause::{
    network::EventEffect,
    types::{Link, Node},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum Candidate {
    Node(Node),
    Link(Link),
}

impl Display for Candidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Candidate::Node(node) => write!(f, "{node}"),
            Candidate::Link(link) => write!(f, "{link}"),
        }
    }
}

pub struct ScoredEvidence {
    pub candidate: Candidate,
    pub evidence: EventEffect,
    pub delta: u16,
}

impl ScoredEvidence {
    pub fn new(can: Candidate, eff: EventEffect, del: u16) -> Self {
        Self {
            candidate: can,
            evidence: eff,
            delta: del,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootCauseTracker {
    scores: HashMap<Candidate, u16>,
    evidence: HashMap<Candidate, Vec<EventEffect>>,
}

impl RootCauseTracker {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            evidence: HashMap::new(),
        }
    }

    pub fn record(&mut self, se: ScoredEvidence) {
        self.scores
            .entry(se.candidate.clone())
            .and_modify(|s| *s += se.delta)
            .or_insert(se.delta);

        self.evidence
            .entry(se.candidate.clone())
            .or_insert_with(Vec::new)
            .push(se.evidence);
    }
}

#[derive(Debug)]
pub struct RootCause {
    candidate: Candidate,
    score: u16,
    evidence: Vec<EventEffect>,
}

impl RootCause {
    pub fn get_cause(tracker: RootCauseTracker) -> Option<Self> {
        let RootCauseTracker { scores, evidence } = tracker;
        let (candidate, score) = scores.into_iter().max_by_key(|(_, score)| *score)?;
        let evidence = evidence.get(&candidate)?.clone();

        Some(Self {
            candidate,
            score,
            evidence,
        })
    }
}

impl Display for RootCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let RootCause {
            candidate,
            score,
            evidence,
        } = self;

        writeln!(f, "Score: {score}")?;
        writeln!(f, "Candidate: {candidate}")?;
        writeln!(f, "Evidence:")?;

        for effect in evidence {
            writeln!(f, "- {effect}\n")?;
        }

        Ok(())
    }
}
