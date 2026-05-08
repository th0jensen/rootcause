use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use serde::Deserialize;

use crate::rootcause::{
    network::EventEffect,
    types::{Link, Node},
};

/// Represents a root cause candidate.
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize)]
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

/// Represents a unit consisting of a root cause [`Candidate`] alongside the
/// evidence ([`EventEffect`]) and scored delta from the baseline.
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

/// Represents a tracker that holds all the root cause [`Candidate`]s for
/// comparison and analysis.
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

    /// Puts the given [`ScoredEvidence`] into the RootCauseTracker as a
    /// record. Adds the delta to the baseline and inserts the provided
    /// evidence ([`EventEffect`]) at the end of the evidence vector. This
    /// function assumes that events are processed in chronological order,
    /// presenting the oldest evidence at the beginning of the vector.
    pub fn record(&mut self, se: &ScoredEvidence) {
        self.scores
            .entry(se.candidate.clone())
            .and_modify(|s| *s += se.delta)
            .or_insert(se.delta);

        self.evidence
            .entry(se.candidate.clone())
            .or_insert_with(Vec::new)
            .push(se.evidence.clone());
    }
}

/// Represents any likely root cause identified in a [`crate::Network`]
/// derived from scored evidence across observed [`crate::Event`]s.
#[derive(Debug)]
pub struct RootCause {
    pub candidate: Candidate,
    pub score: f32,
    pub evidence: Vec<EventEffect>,
}

impl RootCause {
    pub fn get_causes(tracker: RootCauseTracker) -> Option<Vec<Self>> {
        let RootCauseTracker { scores, evidence } = tracker;
        let mut scores = scores.into_iter().collect::<Vec<_>>();
        scores.sort_by(|a, b| b.1.cmp(&a.1));

        let mut causes = Vec::new();
        let total: f32 = scores.iter().map(|(_, s)| *s as f32).sum();
        for score in scores {
            let (candidate, score) = score;
            let score = score as f32 / total;
            let evidence = evidence.get(&candidate)?.clone();

            causes.push(Self {
                candidate,
                score,
                evidence,
            })
        }

        Some(causes)
    }
}

impl Display for RootCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let RootCause {
            candidate,
            score,
            evidence,
        } = self;

        let score = score * 100.0;
        writeln!(f, "Confidence: {score:.2}%")?;
        writeln!(f, "Candidate: {candidate}")?;
        writeln!(f, "Evidence:")?;

        for effect in evidence {
            writeln!(f, "- {effect}\n")?;
        }

        Ok(())
    }
}
