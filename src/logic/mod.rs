use std::{
    fmt::{Display, Write},
    ops::{AddAssign, RangeInclusive},
};

pub type Ptr<T> = Box<T>;

use serde::{Deserialize, Serialize};
use Instruction::*;
use Line::*;
use Logic::*;

mod verify;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Instruction {
    Assumption,                                                  // Implemented
    OrIntroLeft(usize),                                          // Implemented
    OrIntroRight(usize),                                         // Implemented
    OrElim(usize, RangeInclusive<usize>, RangeInclusive<usize>), // Implemented
    NotElim(usize, usize),                                       // Implemented
    NotIntro(RangeInclusive<usize>),                             // Implemented
    BottomElim(usize),                                           // Implemented
    ImplIntro(RangeInclusive<usize>),                            // Implemented
    ImplElim(usize, usize),                                      // Implemented
    AndIntro(usize, usize),                                      // Implemented
    AndElimLeft(usize),                                          // Implemented
    AndElimRight(usize),                                         // Implemented
    Pbc(RangeInclusive<usize>),                                  // Implemented
    Copy(usize),                                                 // Implemented UNSURE IF WANT
    NotNotIntro(usize),                                          // Implemented
    NotNotElim(usize),                                           // Implemented
    Lem,                                                         // Implemented
    Premise,                                                     // Implemented
    Invalid,                                                     // Implemented
}
impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Assumption => write!(f, "ass"),
            OrIntroLeft(i) => write!(f, "∨l {i}"),
            OrIntroRight(i) => write!(f, "∨r {i}"),
            OrElim(o, a, b) => write!(
                f,
                "∨e {o} {}-{} {}-{}",
                a.start(),
                a.end(),
                b.start(),
                b.end()
            ),
            AndIntro(a, b) => write!(f, "∧i {a} {b}"),
            AndElimLeft(i) => write!(f, "∧l {i}"),
            AndElimRight(i) => write!(f, "∧r {i}"),
            NotElim(a, b) => write!(f, "¬e {a} {b}"),
            NotNotElim(a) => write!(f, "¬¬e {a}"),
            NotIntro(a) => write!(f, "¬i {}-{}", a.start(), a.end()),
            NotNotIntro(a) => write!(f, "¬¬i {}", a),
            BottomElim(i) => write!(f, "⊥e {i}"),
            ImplIntro(r) => write!(f, "→i {}-{}", r.start(), r.end()),
            ImplElim(a, b) => write!(f, "→e {a} {b}"),
            Pbc(r) => write!(f, "PBC {}-{}", r.start(), r.end()),
            Copy(i) => write!(f, "copy {i}"),
            Invalid => write!(f, "🛑"),
            Lem => write!(f, "LEM"),
            Premise => write!(f, "pre"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Line<T> {
    Sub(SubProof<T>),
    Log(Ptr<Logic<T>>, Option<Instruction>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectType {
    Term,
    SubProof,
}

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Logic<T> {
    Variable(T),
    And(Ptr<Logic<T>>, Ptr<Logic<T>>),
    Implies(Ptr<Logic<T>>, Ptr<Logic<T>>),
    Not(Ptr<Logic<T>>),
    Or(Ptr<Logic<T>>, Ptr<Logic<T>>),
    Bottom,
    Empty,
}
impl<T> Logic<T> {
    pub fn size(&self) -> usize {
        match self {
            Variable(_) => 1,
            And(a, b) => 1 + a.size() + b.size(),
            Implies(a, b) => 1 + a.size() + b.size(),
            Not(a) => 1 + a.size(),
            Or(a, b) => 1 + a.size() + b.size(),
            Bottom => 1,
            Empty => 0,
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Self> {
        match self {
            Variable(_) => None,
            And(a, _) if index == 0 => Some(a),
            And(_, a) if index == 1 => Some(a),
            And(_, _) => None,
            Implies(a, _) if index == 0 => Some(a),
            Implies(_, a) if index == 1 => Some(a),
            Implies(_, _) => None,
            Not(a) => Some(a),
            Or(a, _) if index == 0 => Some(a),
            Or(_, a) if index == 1 => Some(a),
            Or(_, _) => None,
            Bottom => None,
            Empty => None,
        }
    }

    fn recurse<R>(&mut self, index_map: &[usize], term_func: fn(&mut Logic<T>) -> R) -> Option<R> {
        let Some(index) = index_map.first().copied() else {
            return Some(term_func(self));
        };

        let child = self.get_mut(index)?;
        let res = child.recurse(&index_map[1..], term_func)?;
        Some(res)
    }
}
impl<T: Display> Logic<T> {
    pub fn display(&self, outer: bool) -> String {
        let res = match self {
            Variable(v) => format!("{v}"),
            And(a, b) => format!("{} ∧ {}", a.display(false), b.display(false)),
            Implies(a, b) => format!("{} → {}", a.display(false), b.display(false)),
            Not(a) => format!("¬{}", a.display(false)),
            Or(a, b) => format!("{} ∨ {}", a.display(false), b.display(false)),
            Bottom => "⊥".to_string(),
            Empty => "×".to_string(),
        };
        if outer || matches!(self, Variable(_) | Bottom) {
            res
        } else {
            format!("({res})")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubProof<T>(pub Vec<Line<T>>);
impl<T> Default for SubProof<T> {
    fn default() -> Self {
        Self(vec![Line::Log(Empty.into(), None)])
    }
}
impl<T> SubProof<T> {
    pub fn make_sub_proof(&mut self, index_map: &[usize]) {
        match index_map {
            [i] => {
                if let Some(c) = self.0.get_mut(*i) {
                    *c = Sub(SubProof(vec![Log(Empty.into(), None)]))
                }
            }
            [i, xs @ ..] => {
                if let Some(Sub(s)) = self.0.get_mut(*i) {
                    s.make_sub_proof(xs);
                }
            }
            [] => {}
        }
    }

    pub fn remove_line(&mut self, index_map: &[usize]) {
        match index_map {
            [i] => {
                if *i < self.0.len() {
                    self.0.remove(*i);
                }
            }
            [i, xs @ ..] => {
                if let Some(Sub(s)) = self.0.get_mut(*i) {
                    s.remove_line(xs);
                }
            }
            [] => {}
        }
    }

    pub fn len(&self) -> usize {
        self.0
            .iter()
            .map(|l| match l {
                Sub(s) => s.len(),
                Log(_, _) => 1,
            })
            .sum()
    }

    pub fn recurse<R>(
        &mut self,
        index_map: &[usize],
        sub_func: fn(&mut SubProof<T>) -> R,
        term_func: fn(&mut Logic<T>) -> R,
    ) -> Option<R> {
        let Some(index) = index_map.first().copied() else {
            return Some(sub_func(self));
        };
        let item = self.0.get_mut(index)?;

        match item {
            Line::Sub(s) => s.recurse(&index_map[1..], sub_func, term_func),
            Line::Log(l, _) => l.recurse(&index_map[1..], term_func),
        }
    }

    pub fn stats(&self) -> Stats {
        let mut s = Stats {
            lines: self.0.len(),
            terms: 0,
            sub_proofs: 0,
        };

        for line in &self.0 {
            match line {
                Sub(ns) => {
                    s.sub_proofs += 1;
                    s += ns.stats();
                }
                Log(t, _) => {
                    s.terms += t.size();
                }
            }
        }

        s
    }
}
impl<T: Display> SubProof<T> {
    pub fn display(&self, index: &mut usize, depth: usize) -> String {
        let mut res = String::new();
        for line in &self.0 {
            let mut new_line = String::new();
            match line {
                Sub(sp) => {
                    write!(&mut new_line, "{}", sp.display(index, depth + 1)).unwrap();
                }
                Log(l, inst) => {
                    let l = format!("{index:>3}: {}{}", "│ ".repeat(depth), l.display(true));
                    let len = l.chars().count();
                    let space = if len < 32 { 32 - len } else { 64 - len };
                    writeln!(
                        &mut new_line,
                        "{l}{}{}",
                        " ".repeat(space),
                        inst.clone().map(|s| format!("{s}")).unwrap_or_default()
                    )
                    .unwrap();
                    *index += 1;
                }
            }
            write!(&mut res, "{new_line}").unwrap();
        }
        res
    }
}

pub struct Stats {
    pub terms: usize,
    pub lines: usize,
    pub sub_proofs: usize,
}
impl AddAssign for Stats {
    fn add_assign(&mut self, rhs: Self) {
        self.terms += rhs.terms;
        self.lines += rhs.lines;
        self.sub_proofs += rhs.sub_proofs;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FitchProof<T> {
    #[serde(default)]
    pub proof: SubProof<T>,
    pub prepositions: Vec<Logic<T>>,
    pub result: Ptr<Logic<T>>,
}
impl<T> FitchProof<T> {
    pub fn stats(&self) -> Stats {
        self.proof.stats()
    }

    pub fn next_select(&mut self, input: &[usize]) -> Option<Vec<usize>> {
        if input.is_empty() {
            return None;
        }

        let mut new_input = input.to_vec();
        for i in 0..=1 {
            new_input.push(i);
            if let Some(true) =
                self.proof
                    .recurse(&new_input, |_| false, |t| matches!(&*t, Logic::Empty))
            {
                return Some(new_input);
            }
            new_input.pop();
        }
        let mut new_input = input.to_vec();
        let last = new_input.pop().unwrap();
        if last > 1 {
            self.next_select(&new_input)
        } else {
            new_input.push(last + 1);
            self.next_select(&new_input)
        }
    }
}
impl<T: Display + Clone> Display for FitchProof<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sub_proof = SubProof(
            self.prepositions
                .iter()
                .map(|l| Line::Log(Box::new(l.clone()), Some(Premise)))
                .collect(),
        );
        write!(f, "{}", sub_proof.display(&mut 1, 0))?;

        let proof = self.proof.display(&mut (self.prepositions.len() + 1), 0);
        let result = format!(" result: {}", self.result.display(true));
        let len = proof
            .lines()
            .map(|s| s.chars().count())
            .max()
            .unwrap_or_default()
            .max(result.len());

        write!(f, "{proof}")?;
        writeln!(f, "{}", "─".repeat(len))?;
        write!(f, "{result}")
    }
}

pub fn empty() -> FitchProof<&'static str> {
    FitchProof {
        proof: SubProof(vec![Log(Empty.into(), None)]),
        prepositions: Vec::new(),
        result: Or(
            Variable("p").into(),
            Implies(Variable("p").into(), Variable("q").into()).into(),
        )
        .into(), // p ∨ (p → q)
    }
}
