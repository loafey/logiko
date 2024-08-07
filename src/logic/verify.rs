use std::{collections::HashMap, hash::Hash};

use super::{FitchProof, Instruction, Line, Logic, SubProof};

#[derive(Clone)]
struct State<T> {
    can_assume: bool,
    symbols: HashMap<Logic<T>, Option<Logic<T>>>,
}
impl<T: std::fmt::Debug + std::fmt::Display> std::fmt::Debug for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}}}",
            self.symbols
                .iter()
                .map(|(k, v)| format!(
                    "{}: {:?}",
                    k.display(true),
                    v.as_ref().map(|k| k.display(true))
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
impl<T> State<T> {
    pub fn can_assume(mut self) -> Self {
        self.can_assume = true;
        self
    }
}
impl<T> Default for State<T> {
    fn default() -> Self {
        Self {
            can_assume: true,
            symbols: HashMap::new(),
        }
    }
}
fn find_symbol<'a, T: Hash + Eq + PartialEq>(
    nk: &Logic<T>,
    nv: &Option<Logic<T>>,
    state: &'a [State<T>],
) -> Option<(&'a Logic<T>, &'a Option<Logic<T>>)> {
    for s in state.iter().rev() {
        if let Some((k, v)) = s.symbols.get_key_value(nk) {
            if nv.is_some() {
                if v == nv {
                    return Some((k, v));
                }
            } else {
                return Some((k, v));
            }
        }
    }
    None
}

impl<T: Clone + Hash + Eq + std::fmt::Display + std::fmt::Debug> SubProof<T> {
    fn verify(&mut self, mut stack: Vec<State<T>>) -> (Option<Logic<T>>, Option<Logic<T>>) {
        let mut first = None;
        let mut last = None;
        let proof_len = self.0.len();
        for (i, line) in self.0.iter_mut().enumerate() {
            let is_first = i == 0;
            match line {
                Line::Sub(s) => {
                    stack.last_mut().unwrap().can_assume = false;
                    let mut new_stack = stack.clone();
                    new_stack.push(State::default().can_assume());
                    let (f, l) = s.verify(new_stack);
                    if let Some(f) = f {
                        stack.last_mut().unwrap().symbols.insert(f, l);
                    }
                }
                Line::Log(l, t) => {
                    // println!("Checking: {}\t| State: {stack:?}", l.display(true));
                    if first.is_none() {
                        first = Some(l.as_ref().clone());
                    }
                    if last.is_none() && i + 1 == proof_len {
                        last = Some(l.as_ref().clone());
                    }

                    if is_first && stack.last_mut().unwrap().can_assume {
                        *t = Some(Instruction::Assumption);
                        stack
                            .last_mut()
                            .unwrap()
                            .symbols
                            .insert((**l).clone(), None);
                        continue;
                    }

                    // Bottom elim
                    if find_symbol(&Logic::Bottom, &None, &stack).is_some() {
                        *t = Some(Instruction::BottomElim(0));
                    }
                    // PBC
                    else if find_symbol(&Logic::Not(l.clone()), &Some(Logic::Bottom), &stack)
                        .is_some()
                    {
                        *t = Some(Instruction::Pbc(0..=0));
                    } else {
                        match &mut **l {
                            // Impl introduction
                            Logic::Implies(a, b) => {
                                if find_symbol(&**a, &Some((**b).clone()), &stack).is_some() {
                                    *t = Some(Instruction::ImplIntro(0..=0));
                                } else {
                                    *t = Some(Instruction::Invalid);
                                }
                            }
                            // Or introduction
                            Logic::Or(a, b) => {
                                *t = Some(if find_symbol(&*a, &None, &stack).is_some() {
                                    Instruction::OrIntroLeft(0)
                                } else if find_symbol(&*b, &None, &stack).is_some() {
                                    Instruction::OrIntroRight(0)
                                } else {
                                    Instruction::Invalid
                                });
                            }
                            // Not elimination
                            Logic::Bottom => {
                                let mut valid = false;
                                'outer: for s in &stack {
                                    for term in s.symbols.keys() {
                                        if find_symbol(
                                            &Logic::Not(term.clone().into()),
                                            &None,
                                            &stack,
                                        )
                                        .is_some()
                                        {
                                            *t = Some(Instruction::NotElim(0, 0));
                                            valid = true;
                                            break 'outer;
                                        }
                                    }
                                }
                                //stack.last_mut().unwrap()
                                if !valid {
                                    *t = Some(Instruction::Invalid)
                                }
                            }
                            x => println!("ERROR: {x:?}"),
                        }
                    }
                    stack
                        .last_mut()
                        .unwrap()
                        .symbols
                        .insert((**l).clone(), None);
                }
            }
        }

        (first, last)
    }

    fn has_invalid(&self) -> bool {
        let mut res = true;
        for l in &self.0 {
            match l {
                Line::Sub(s) => res &= s.has_invalid(),
                Line::Log(_, t) => {
                    res &= t
                        .as_ref()
                        .map(|a| *a == Instruction::Invalid)
                        .unwrap_or_default()
                }
            }
        }
        res
    }
}
impl<T: Clone + Hash + Eq + std::fmt::Display + std::fmt::Debug> FitchProof<T> {
    pub fn verify(&mut self) -> bool {
        self.proof.verify(vec![State::default()]);
        !self.proof.has_invalid()
    }
}
