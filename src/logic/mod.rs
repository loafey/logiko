use std::{
    fmt::{Display, Write},
    ops::RangeInclusive,
};

pub type Ptr<T> = Box<T>;

use Instruction::*;
use Line::*;
use Logic::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Assumption,
    OrIntroLeft(usize),
    OrIntroRight(usize),
    OrElim(RangeInclusive<usize>, RangeInclusive<usize>),
    NotElim(usize, usize),
    BottomElim(usize),
    ImplIntro(RangeInclusive<usize>),
    AndIntro(usize, usize),
    AndElimLeft(usize),
    AndElimRight(usize),
    Pbc(RangeInclusive<usize>),
    Invalid,
}
impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Assumption => write!(f, "ass"),
            OrIntroLeft(i) => write!(f, "∨l {i}"),
            OrIntroRight(i) => write!(f, "∨r {i}"),
            OrElim(a, b) => write!(f, "∨e {}-{} {}-{}", a.start(), a.end(), b.start(), b.end()),
            AndIntro(a, b) => write!(f, "∧i {a} {b}"),
            AndElimLeft(i) => write!(f, "∧l {i}"),
            AndElimRight(i) => write!(f, "∧r {i}"),
            NotElim(a, b) => write!(f, "¬e {a} {b}"),
            BottomElim(i) => write!(f, "⊥e {i}"),
            ImplIntro(r) => write!(f, "→i {}-{}", r.start(), r.end()),
            Pbc(r) => write!(f, "PBC {}-{}", r.start(), r.end()),
            Invalid => write!(f, "🛑"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line<T> {
    Sub(SubProof<T>),
    Log(Ptr<Logic<T>>, Option<Instruction>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectType {
    Term,
    SubProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Logic<T> {
    Variable(T),
    And(Ptr<Logic<T>>, Ptr<Logic<T>>),
    Implies(Ptr<Logic<T>>, Ptr<Logic<T>>),
    Equivalent(Ptr<Logic<T>>, Ptr<Logic<T>>),
    Not(Ptr<Logic<T>>),
    Or(Ptr<Logic<T>>, Ptr<Logic<T>>),
    Bottom,
    Empty,
}
impl<T> Logic<T> {
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Self> {
        match self {
            Variable(_) => None,
            And(a, _) if index == 0 => Some(a),
            And(_, a) if index == 1 => Some(a),
            And(_, _) => None,
            Implies(a, _) if index == 0 => Some(a),
            Implies(_, a) if index == 1 => Some(a),
            Implies(_, _) => None,
            Equivalent(a, _) if index == 0 => Some(a),
            Equivalent(_, a) if index == 1 => Some(a),
            Equivalent(_, _) => None,
            Not(a) => Some(a),
            Or(a, _) if index == 0 => Some(a),
            Or(_, a) if index == 1 => Some(a),
            Or(_, _) => None,
            Bottom => None,
            Empty => None,
        }
    }
}
impl<T> Logic<T> {
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
    fn display(&self, outer: bool) -> String {
        let res = match self {
            Variable(v) => format!("{v}"),
            And(a, b) => format!("{} ∧ {}", a.display(false), b.display(false)),
            Implies(a, b) => format!("{} → {}", a.display(false), b.display(false)),
            Equivalent(a, b) => format!("{} = {}", a.display(false), b.display(false)),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubProof<T>(pub Vec<Line<T>>);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FitchProof<T> {
    pub proof: SubProof<T>,
    pub prepositions: Vec<Ptr<Logic<T>>>,
    pub result: Ptr<Logic<T>>,
}
impl<T> FitchProof<T> {
    pub fn verify(&mut self) -> bool {
        false
    }
}
impl<T: Display> Display for FitchProof<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let proof = self.proof.display(&mut 1, 0);
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

pub fn example_proof() -> FitchProof<&'static str> {
    let vec = vec![
        Sub(SubProof(vec![
            // 1. | ¬(p ∨ (p → q)) assumption
            Log(
                Not(Or(
                    Variable("p").into(),
                    Implies(Variable("p").into(), Variable("q").into()).into(),
                )
                .into())
                .into(),
                None,
                // Assumption,
            ),
            Sub(SubProof(vec![
                // 2. | | p              assumption
                Log(
                    Variable("p").into(),
                    None,
                    // Assumption
                ),
                // 3. | | p ∨ (p → q)    ∨i1 2
                Log(
                    (Or(
                        (Variable("p")).into(),
                        (Implies((Variable("p")).into(), (Variable("q")).into())).into(),
                    ))
                    .into(),
                    None,
                    // OrIntroLeft(2),
                ),
                // 4. | | ⊥              ¬e (1,3)
                Log(
                    (Bottom).into(),
                    None,
                    // NotElim(1, 3)
                ),
                // 5. | | q              ⊥e 4
                Log(
                    (Variable("q")).into(),
                    None,
                    // BottomElim(4)
                ),
            ])),
            // 6. | p → q          →i (2–5)
            Log(
                (Implies((Variable("p")).into(), (Variable("q")).into())).into(),
                None,
                // ImplIntro(2..=5),
            ),
            // 7. | p ∨ (p → q)    ∨i2 6
            Log(
                (Or(
                    (Variable("p")).into(),
                    (Implies((Variable("p")).into(), (Variable("q")).into())).into(),
                ))
                .into(),
                None,
                // OrIntroRight(6),
            ),
            // 8. | ⊥              ¬e (1,7)
            Log(
                (Bottom).into(),
                // NotElim(1, 7)
                None,
            ),
        ])),
        // 9. p ∨ (p → q)    PBC (1–8)
        Log(
            (Or(
                (Variable("p")).into(),
                (Implies((Variable("p")).into(), (Variable("q")).into())).into(),
            ))
            .into(),
            // Pbc(1..=8),
            None,
        ),
    ];
    FitchProof {
        proof: SubProof(vec),
        prepositions: Vec::new(),
        result: (Or(
            (Variable("p")).into(),
            (Implies((Variable("p")).into(), (Variable("q")).into())).into(),
        ))
        .into(), // p ∨ (p → q)
    }
}
