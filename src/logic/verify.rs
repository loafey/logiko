use super::{FitchProof, Instruction, Line, Logic, SubProof};
use crate::util::Droppable;
use itertools::Itertools;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

#[derive(Debug, Eq, Clone)]
struct Position<T> {
    index: usize,
    logic: Logic<T>,
}
impl<T> From<Logic<T>> for Position<T> {
    fn from(val: Logic<T>) -> Self {
        Position {
            index: 0,
            logic: val,
        }
    }
}

impl<T: PartialEq> PartialEq for Position<T> {
    fn eq(&self, other: &Self) -> bool {
        self.logic == other.logic
    }
}

impl<T: Hash> Hash for Position<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.logic.hash(state);
    }
}

#[derive(Clone)]
struct State<T> {
    can_assume: bool,
    symbols: HashMap<Position<T>, Option<Position<T>>>,
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
            can_assume: false,
            symbols: HashMap::new(),
        }
    }
}

fn find_implication_elim<'a, T: Hash + Eq + PartialEq + Clone>(
    nk: &Position<T>,
    state: &'a [State<T>],
) -> Option<(&'a Position<T>, &'a Position<T>)> {
    let mut valid_impls = Vec::new();
    for s in &state.iter().rev().collect::<Vec<_>>() {
        for (k, r) in s.symbols.iter() {
            if r.is_none() {
                if let Logic::Implies(_, b) = &k.logic {
                    if **b == nk.logic {
                        valid_impls.push(k);
                    }
                }
            }
        }
    }

    for left in valid_impls {
        if let Some((s, _)) = find_symbol(
            &Position {
                index: 0,
                logic: left.logic.clone(),
            },
            &None,
            state,
        ) {
            return Some((left, s));
        }
    }
    None
}
#[allow(clippy::type_complexity)]
fn find_or_elim<'a, T: Hash + Eq + PartialEq>(
    nk: &Position<T>,
    state: &'a [State<T>],
) -> Option<(
    &'a Position<T>, // or
    &'a Position<T>, // left or sub proof start
    &'a Position<T>, // left or sub proof end
    &'a Position<T>, // right or sub proof start
    &'a Position<T>, // right or sub proof start
)> {
    let mut valid_subs = Vec::new();
    let mut ors = Vec::new();
    for s in &state.iter().rev().collect::<Vec<_>>() {
        for (k, r) in s.symbols.iter() {
            if let Some(r) = r {
                if r == nk {
                    valid_subs.push((k, r))
                }
            } else if let Logic::Or(a, b) = &k.logic {
                ors.push((k, a, b))
            }
        }
    }

    for v in valid_subs.iter().permutations(2) {
        let [(a, a_end), (b, b_end)] = &v[..] else {
            unreachable!()
        };

        for (o, oa, ob) in &ors {
            if a.logic == ***oa && b.logic == ***ob {
                return Some((*o, a, a_end, b, b_end));
            }
        }
    }

    None
}
fn find_symbol_in_and<'a, T: Hash + Eq + PartialEq>(
    nk: &Position<T>,
    state: &'a [State<T>],
) -> Option<(bool, &'a Position<T>)> {
    for s in state.iter().rev() {
        for (k, _) in s.symbols.iter() {
            match &k.logic {
                Logic::And(lk, _) if nk.logic == **lk => return Some((true, k)),
                Logic::And(_, lk) if nk.logic == **lk => return Some((false, k)),
                _ => continue,
            }
        }
    }
    None
}
fn find_symbol<'a, T: Hash + Eq + PartialEq>(
    nk: &Position<T>,
    nv: &Option<Position<T>>,
    state: &'a [State<T>],
) -> Option<(&'a Position<T>, &'a Option<Position<T>>)> {
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

impl<T: Clone + Hash + Eq + Debug + Display> SubProof<T> {
    #[allow(clippy::type_complexity)]
    fn verify(
        &mut self,
        index: &mut usize,
        mut stack: Vec<State<T>>,
    ) -> Result<(Option<Position<T>>, Option<Position<T>>), String> {
        let mut first = None;
        let mut last = None;
        let mut error_log = String::new();
        let proof_len = self.0.len();
        for (i, line) in self.0.iter_mut().enumerate() {
            let is_first = i == 0;
            match line {
                Line::Sub(s) => {
                    stack.last_mut().unwrap().can_assume = false;
                    let mut new_stack = stack.clone();
                    new_stack.push(State::default().can_assume());
                    let (f, l) = s.verify(index, new_stack)?;
                    if let Some(f) = f {
                        stack.last_mut().unwrap().symbols.insert(f, l);
                    }
                }
                Line::Log(l, t) => {
                    *index += 1;
                    // println!("Checking: {}\t| State: {stack:?}", l.display(true));
                    if first.is_none() {
                        first = Some(Position {
                            index: *index,
                            logic: l.as_ref().clone(),
                        });
                    }
                    if last.is_none() && i + 1 == proof_len {
                        last = Some(Position {
                            index: *index,
                            logic: l.as_ref().clone(),
                        });
                    }

                    if is_first && stack.last_mut().unwrap().can_assume {
                        *t = Some(Instruction::Assumption);
                        stack.last_mut().unwrap().symbols.insert(
                            Position {
                                index: *index,
                                logic: (**l).clone(),
                            },
                            None,
                        );
                        continue;
                    }

                    if matches!(**l, Logic::Empty) {
                        *t = Some(Instruction::Invalid);
                    } else if let Some((a, _)) = find_symbol(
                        &Position {
                            index: 0,
                            logic: Logic::Not(Logic::Not(l.clone()).into()),
                        },
                        &None,
                        &stack,
                    ) {
                        *t = Some(Instruction::NotNotElim(a.index));
                    }
                    // Impl elim
                    else if let Some((a, b)) = find_implication_elim(
                        &Position {
                            logic: *l.clone(),
                            index: 0,
                        },
                        &stack,
                    ) {
                        *t = Some(Instruction::ImplElim(a.index, b.index));
                    }
                    // Bottom elim
                    else if let Some((a, _)) = find_symbol(&Logic::Bottom.into(), &None, &stack) {
                        *t = Some(Instruction::BottomElim(a.index));
                    }
                    // and elim
                    else if let Some((left, a)) = find_symbol_in_and(
                        &Position {
                            logic: *l.clone(),
                            index: 0,
                        },
                        &stack,
                    ) {
                        if left {
                            *t = Some(Instruction::AndElimLeft(a.index))
                        } else {
                            *t = Some(Instruction::AndElimRight(a.index))
                        }
                    }
                    // or elim
                    else if let Some((o, a, a_end, b, b_end)) = find_or_elim(
                        &Position {
                            logic: *l.clone(),
                            index: 0,
                        },
                        &stack,
                    ) {
                        *t = Some(Instruction::OrElim(
                            o.index,
                            a.index..=a_end.index,
                            b.index..=b_end.index,
                        ))
                    }
                    // PBC
                    else if let Some((a, b)) = find_symbol(
                        &Logic::Not(l.clone()).into(),
                        &Some(Logic::Bottom.into()),
                        &stack,
                    ) {
                        *t = Some(Instruction::Pbc(a.index..=b.as_ref().unwrap().index));
                    } else {
                        match &mut **l {
                            // Not Not intro
                            Logic::Not(a) if matches!(&**a, Logic::Not(_)) => {
                                let Logic::Not(a) = &**a else { unreachable!() };
                                if let Some((a, _)) =
                                    find_symbol(&(**a).clone().into(), &None, &stack)
                                {
                                    *t = Some(Instruction::NotNotIntro(a.index));
                                } else {
                                    *t = Some(Instruction::Invalid);
                                }
                            }
                            // Not intro
                            Logic::Not(a) => {
                                if let Some((a, b)) = find_symbol(
                                    &(**a).clone().into(),
                                    &Some(Logic::Bottom.into()),
                                    &stack,
                                ) {
                                    *t = Some(Instruction::NotIntro(
                                        a.index..=b.clone().map(|b| b.index).unwrap_or_default(),
                                    ));
                                } else {
                                    *t = Some(Instruction::Invalid);
                                }
                            }
                            // And intro
                            Logic::And(a, b) => {
                                if let Some((na, nb)) = find_symbol(
                                    &Position {
                                        index: 0,
                                        logic: *a.clone(),
                                    },
                                    &None,
                                    &stack,
                                )
                                .and_then(|(a, _)| {
                                    find_symbol(
                                        &Position {
                                            index: 0,
                                            logic: *b.clone(),
                                        },
                                        &None,
                                        &stack,
                                    )
                                    .map(|(b, _)| (a, b))
                                }) {
                                    *t = Some(Instruction::AndIntro(na.index, nb.index));
                                }
                            }
                            // Impl introduction
                            Logic::Implies(a, b) => {
                                if let Some((a, b)) = find_symbol(
                                    &(**a).clone().into(),
                                    &Some((**b).clone().into()),
                                    &stack,
                                ) {
                                    *t = Some(Instruction::ImplIntro(
                                        a.index..=b.as_ref().unwrap().index,
                                    ));
                                } else {
                                    *t = Some(Instruction::Invalid);
                                }
                            }
                            // Or introduction
                            Logic::Or(a, b) => {
                                if **a == Logic::Not((*b).clone()) {
                                    // LEM
                                    *t = Some(Instruction::Lem);
                                } else {
                                    *t = Some(
                                        if let Some((p, _)) =
                                            find_symbol(&(**a).clone().into(), &None, &stack)
                                        {
                                            Instruction::OrIntroLeft(p.index)
                                        } else if let Some((p, _)) =
                                            find_symbol(&(**b).clone().into(), &None, &stack)
                                        {
                                            Instruction::OrIntroRight(p.index)
                                        } else {
                                            Instruction::Invalid
                                        },
                                    );
                                }
                            }
                            // Not elimination
                            Logic::Bottom => {
                                let mut valid = false;
                                'outer: for s in &stack {
                                    for term in s.symbols.keys() {
                                        if let Some((n_term, _)) = find_symbol(
                                            &Logic::Not(term.logic.clone().into()).into(),
                                            &None,
                                            &stack,
                                        ) {
                                            *t = Some(Instruction::NotElim(
                                                term.index,
                                                n_term.index,
                                            ));
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
                            Logic::Variable(_) => {
                                *t = Some(Instruction::Invalid);
                            }
                            x => {
                                *t = Some(Instruction::Invalid);
                                error_log.push_str(&format!(
                                    "ERROR: Failed to find suitable rule for term \"{}\"\n",
                                    x.display(true)
                                ));
                            }
                        }
                    }
                    stack.last_mut().unwrap().symbols.insert(
                        Position {
                            index: *index,
                            logic: (**l).clone(),
                        },
                        None,
                    );
                }
            }
        }

        if error_log.is_empty() {
            Ok((first, last))
        } else {
            Err(error_log)
        }
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
impl<T: Clone + Hash + Eq + Debug + Display> FitchProof<T> {
    pub fn verify(&mut self) -> Result<bool, String> {
        let mut state = State::default();
        self.prepositions.iter().enumerate().for_each(|(i, l)| {
            state
                .symbols
                .insert(
                    Position {
                        logic: l.clone(),
                        index: i + 1,
                    },
                    None,
                )
                .drop()
        });

        self.proof
            .verify(&mut self.prepositions.len(), vec![state])?;
        Ok(!self.proof.has_invalid()
            && self
                .proof
                .0
                .last()
                .map(|l| match l {
                    Line::Sub(_) => false,
                    Line::Log(l, _) => **l == *self.result,
                })
                .unwrap_or_default())
    }
}
