use std::{
    fmt::{Display, Write as _},
    ops::RangeInclusive,
};
use Instruction::*;
use Line::*;
use Logic::*;

type Propositional = ();

enum Instruction {
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

enum Line<T> {
    Sub(SubProof<T>),
    Logic(Logic<T>, Instruction),
}

enum Logic<T> {
    Variable(T),
    And(Box<Logic<T>>, Box<Logic<T>>),
    Implies(Box<Logic<T>>, Box<Logic<T>>),
    Equivalent(Box<Logic<T>>, Box<Logic<T>>),
    Not(Box<Logic<T>>),
    Or(Box<Logic<T>>, Box<Logic<T>>),
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

struct SubProof<T>(Vec<Line<T>>);
impl<T: Display> SubProof<T> {
    pub fn display(&self, index: &mut usize, depth: usize) -> String {
        let mut res = String::new();
        for line in &self.0 {
            let mut new_line = String::new();
            match line {
                Sub(sp) => {
                    write!(&mut new_line, "{}", sp.display(index, depth + 1)).unwrap();
                }
                Logic(l, inst) => {
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

struct FitchProof<T> {
    pub proof: SubProof<T>,
    pub result: Logic<T>,
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

fn main() {
    let proof = FitchProof {
        proof: SubProof(vec![
            Sub(SubProof(vec![
                // 1. | ¬(p ∨ (p → q)) assumption
                Logic(
                    Not(Box::new(Or(
                        Box::new(Variable("p")),
                        Box::new(Implies(Box::new(Variable("p")), Box::new(Variable("q")))),
                    ))),
                    Assumption,
                ),
                Sub(SubProof(vec![
                    // 2. | | p              assumption
                    Logic(Variable("p"), Assumption),
                    // 3. | | p ∨ (p → q)    ∨i1 2
                    Logic(
                        Or(
                            Box::new(Variable("p")),
                            Box::new(Implies(Box::new(Variable("p")), Box::new(Variable("q")))),
                        ),
                        OrIntroLeft(2),
                    ),
                    // 4. | | ⊥              ¬e (1,3)
                    Logic(Bottom, NotElim(1, 3)),
                    // 5. | | q              ⊥e 4
                    Logic(Variable("q"), BottomElim(4)),
                ])),
                // 6. | p → q          →i (2–5)
                Logic(
                    Implies(Box::new(Variable("p")), Box::new(Variable("q"))),
                    ImplIntro(2..=5),
                ),
                // 7. | p ∨ (p → q)    ∨i2 6
                Logic(
                    Or(
                        Box::new(Variable("p")),
                        Box::new(Implies(Box::new(Variable("p")), Box::new(Variable("q")))),
                    ),
                    OrIntroRight(6),
                ),
                // 8. | ⊥              ¬e (1,7)
                Logic(Bottom, NotElim(1, 7)),
            ])),
            // 9. p ∨ (p → q)    PBC (1–8)
            Logic(
                Or(
                    Box::new(Variable("p")),
                    Box::new(Implies(Box::new(Variable("p")), Box::new(Variable("q")))),
                ),
                Pbc(1..=8),
            ),
        ]),
        result: Or(
            Box::new(Variable("p")),
            Box::new(Implies(Box::new(Variable("p")), Box::new(Variable("q")))),
        ), // p ∨ (p → q)
    };
    println!("{proof}");
}
