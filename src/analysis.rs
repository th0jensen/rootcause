use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use serde::Deserialize;

use crate::types::{Event, EventType, Link, Node};

const LINK_DOWN: u16 = 1;
const UNREACHABLE_AGREE: u16 = 0;
const UNREACHABLE_DISAGREE: u16 = 1;

/// Represents the effect of an [`Event`] on a [`crate::Network`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub enum EventEffect {
    TopologyChanged { event: Event },
    Observation { event: Event, reachable: bool },
    Ignored,
}

impl EventEffect {
    /// Evaluate this [`EventEffect`] as a potential root cause candidate for
    /// [`crate::RootCauseTracker`].
    pub fn candidate(&self) -> Option<Vec<ScoredEvidence>> {
        match self {
            Self::Ignored => None,
            Self::TopologyChanged { event } => self.on_topology(event),
            Self::Observation { event, reachable } => self.on_observation(event, reachable),
        }
    }

    /// Score an [`EventType::LinkDown`] topology change as evidence against
    /// the affected [`Link`] and target [`Node`].
    pub fn on_topology(&self, event: &Event) -> Option<Vec<ScoredEvidence>> {
        if matches!(event.event_type, EventType::LinkDown) {
            let t = event.target.clone();
            let l = Link::new(event.node.clone(), t.clone());
            let s = LINK_DOWN;
            Some(vec![
                ScoredEvidence::new(Candidate::Link(l), self.clone(), s),
                ScoredEvidence::new(Candidate::Node(t), self.clone(), s),
            ])
        } else {
            None
        }
    }

    /// Score an [`EventType::NodeUnreachable`] observation as evidence against
    /// the target [`Node`]. A higher delta is assigned when the topology
    /// confirms the node is still reachable, suggesting the report is
    /// unexpected and more significant.
    pub fn on_observation(&self, event: &Event, reachable: &bool) -> Option<Vec<ScoredEvidence>> {
        Some(vec![ScoredEvidence::new(
            Candidate::Node(event.target.clone()),
            self.clone(),
            if *reachable {
                UNREACHABLE_DISAGREE
            } else {
                UNREACHABLE_AGREE
            },
        )])
    }
}

impl Display for EventEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventEffect::TopologyChanged { event } => write!(f, "{event}"),
            EventEffect::Observation { event, reachable } => {
                write!(f, "{event}\n  Topology still reachable: {reachable}")
            }
            EventEffect::Ignored => write!(f, ""),
        }
    }
}

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

/// Represents any likely root cause identified in a [`crate::Network`]
/// derived from scored evidence across observed [`crate::Event`]s.
#[derive(Debug, Clone)]
pub struct RootCause {
    pub candidate: Candidate,
    pub score: f32,
    pub evidence: Vec<EventEffect>,
}

impl Display for RootCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let RootCause {
            candidate,
            score,
            evidence,
        } = self;

        let score = score * 100.0;
        if score > 0.0 {
            writeln!(f, "Confidence: {score:.2}%")?;
        }
        writeln!(f, "Candidate: {candidate}")?;
        writeln!(f, "Evidence:")?;

        for effect in evidence {
            writeln!(f, "- {effect}\n")?;
        }

        Ok(())
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
        let score = |s: &mut u16| *s += se.delta;

        self.scores
            .entry(se.candidate.clone())
            .and_modify(score)
            .or_insert(se.delta);

        self.evidence
            .entry(se.candidate.clone())
            .or_insert_with(Vec::new)
            .push(se.evidence.clone());
    }

    /// Gets all likely causes from the RootCauseTracker, processing them into
    /// the [`RootCauseResult`] struct. Results will be sorted primarily by
    /// score and secondarily by [`Candidate`].
    pub fn get_causes(&self) -> anyhow::Result<RootCauseResult> {
        let RootCauseTracker { scores, evidence } = self.clone();
        let mut scores = scores.into_iter().collect::<Vec<_>>();
        scores.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        let mut causes = Vec::new();
        let total: f32 = scores.iter().map(|(_, s)| *s as f32).sum();
        for (candidate, score) in scores {
            let score = score as f32 / total;
            let evidence = evidence.get(&candidate);
            match evidence {
                None => continue,
                Some(e) => causes.push(RootCause {
                    candidate,
                    score,
                    evidence: e.clone(),
                }),
            }
        }

        RootCauseResult::new(causes)
    }
}

pub struct RootCauseResult {
    pub most_likely: Vec<RootCause>,
    pub less_likely: Vec<RootCause>,
    pub symptoms: Vec<RootCause>,
}

impl RootCauseResult {
    /// Creates a new RootCauseResult from a given Vec<[`RootCause`]>. This
    /// function assumes that elements are already ordered.
    pub fn new(causes: Vec<RootCause>) -> anyhow::Result<Self> {
        let top_score = causes
            .first()
            .map(|c| c.score)
            .ok_or_else(|| anyhow::anyhow!("no causes found for network"))?;

        Ok(Self {
            most_likely: causes
                .iter()
                .filter(|c| c.score == top_score)
                .map(Clone::clone)
                .collect::<Vec<_>>(),
            less_likely: causes
                .iter()
                .filter(|c| c.score != top_score && c.score > 0.0)
                .map(Clone::clone)
                .collect::<Vec<_>>(),
            symptoms: causes
                .iter()
                .filter(|c| c.score == 0.0)
                .map(Clone::clone)
                .collect::<Vec<_>>(),
        })
    }
}

impl Display for RootCauseResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let plural = |v: &Vec<RootCause>| if v.len() > 1 { "s" } else { "" };
        if !self.most_likely.is_empty() {
            writeln!(f, "Most likely root cause{}:\n", plural(&self.most_likely))?;
            for cause in &self.most_likely {
                writeln!(f, "{cause}")?;
            }
        }
        if !self.less_likely.is_empty() {
            writeln!(f, "Less likely root cause{}:\n", plural(&self.less_likely))?;
            for cause in &self.less_likely {
                writeln!(f, "{cause}")?;
            }
        }
        if !self.symptoms.is_empty() {
            writeln!(
                f,
                "Observed downstream symptom{}:\n",
                plural(&self.symptoms)
            )?;
            for cause in &self.symptoms {
                writeln!(f, "{cause}")?;
            }
        }

        Ok(())
    }
}
