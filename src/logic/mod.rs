use std::{
    fmt::{Display, Write},
    ops::RangeInclusive,
    sync::Arc,
};

use Instruction::*;
use Line::*;
use Logic::*;
type Propositional = ();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Assumption,
    OrIntroLeft(usize),
    OrIntroRight(usize),
    NotElim(usize, usize),
    BottomElim(usize),
    ImplIntro(RangeInclusive<usize>),
    Pbc(RangeInclusive<usize>),
}
impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Assumption => write!(f, "ass"),
            OrIntroLeft(i) => write!(f, "∧l {i}"),
            OrIntroRight(i) => write!(f, "∧r {i}"),
            NotElim(a, b) => write!(f, "¬e {a} {b}"),
            BottomElim(i) => write!(f, "⊥e {i}"),
            ImplIntro(r) => write!(f, "→i {}-{}", r.start(), r.end()),
            Pbc(r) => write!(f, "PBC {}-{}", r.start(), r.end()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line<T> {
    Sub(SubProof<T>),
    Log(Arc<Logic<T>>, Instruction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Logic<T> {
    Variable(T),
    And(Arc<Logic<T>>, Arc<Logic<T>>),
    Implies(Arc<Logic<T>>, Arc<Logic<T>>),
    Equivalent(Arc<Logic<T>>, Arc<Logic<T>>),
    Not(Arc<Logic<T>>),
    Or(Arc<Logic<T>>, Arc<Logic<T>>),
    Bottom,
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
        };
        if outer || matches!(self, Variable(_) | Bottom) {
            res
        } else {
            format!("({res})")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubProof<T>(Vec<Line<T>>);
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
                    writeln!(&mut new_line, "{l}{}{inst}", " ".repeat(space)).unwrap();
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
    pub result: Arc<Logic<T>>,
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
                Assumption,
            ),
            Sub(SubProof(vec![
                // 2. | | p              assumption
                Log(Variable("p").into(), Assumption),
                // 3. | | p ∨ (p → q)    ∨i1 2
                Log(
                    Or(
                        Variable("p").into(),
                        Implies(Variable("p").into(), Variable("q").into()).into(),
                    )
                    .into(),
                    OrIntroLeft(2),
                ),
                // 4. | | ⊥              ¬e (1,3)
                Log(Bottom.into(), NotElim(1, 3)),
                // 5. | | q              ⊥e 4
                Log(Variable("q").into(), BottomElim(4)),
            ])),
            // 6. | p → q          →i (2–5)
            Log(
                Implies(Variable("p").into(), Variable("q").into()).into(),
                ImplIntro(2..=5),
            ),
            // 7. | p ∨ (p → q)    ∨i2 6
            Log(
                Or(
                    Variable("p").into(),
                    Implies(Variable("p").into(), Variable("q").into()).into(),
                )
                .into(),
                OrIntroRight(6),
            ),
            // 8. | ⊥              ¬e (1,7)
            Log(Bottom.into(), NotElim(1, 7)),
        ])),
        // 9. p ∨ (p → q)    PBC (1–8)
        Log(
            Or(
                Variable("p").into(),
                Implies(Variable("p").into(), Variable("q").into()).into(),
            )
            .into(),
            Pbc(1..=8),
        ),
    ];
    let vec = vec;
    FitchProof {
        proof: SubProof(vec),
        result: Or(
            Variable("p").into(),
            Implies(Variable("p").into(), Variable("q").into()).into(),
        )
        .into(), // p ∨ (p → q)
    }
}
