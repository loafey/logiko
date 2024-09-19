use super::{FitchProof, Instruction, Line, Logic, SubProof};
use itertools::Itertools;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
};

#[derive(Clone)]
struct State<T> {
    can_assume: bool,
    #[allow(clippy::type_complexity)]
    symbols: HashMap<(Logic<T>, Option<Logic<T>>), (usize, usize)>,
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

fn find_impl_elim<T: Hash + Eq + PartialEq + Clone>(
    nk: &Logic<T>,
    state: &[State<T>],
) -> Option<(usize, usize)> {
    let mut valid_impls = Vec::new();
    for s in &state.iter().rev().collect::<Vec<_>>() {
        for ((k, s), (r, _)) in s.symbols.iter() {
            if s.is_none() {
                if let Logic::Implies(a, b) = &k {
                    if **b == *nk {
                        valid_impls.push((a, r));
                    }
                }
            }
        }
    }

    for (left, r) in valid_impls {
        if let Some((_, (p, _))) = find_symbol(&(*left.clone(), None), state) {
            return Some((p, *r));
        }
    }
    None
}
#[allow(clippy::type_complexity)]
fn find_or_elim<T: Hash + Eq + PartialEq>(
    nk: &Logic<T>,
    state: &[State<T>],
) -> Option<(
    usize, // or
    usize, // left or sub proof start
    usize, // left or sub proof end
    usize, // right or sub proof start
    usize, // right or sub proof start
)> {
    let mut valid_subs = Vec::new();
    let mut ors = Vec::new();
    for s in &state.iter().rev().collect::<Vec<_>>() {
        for ((k, r), (p1, p2)) in s.symbols.iter() {
            if let Some(r) = r {
                if r == nk {
                    valid_subs.push(((k, r), (p1, p2)))
                }
            } else if let Logic::Or(a, b) = &k {
                ors.push((p1, a, b))
            }
        }
    }

    for v in valid_subs.iter().permutations(2) {
        let [((a, _), (a_start, a_end)), ((b, _), (b_start, b_end))] = &v[..] else {
            unreachable!()
        };

        for (or_pos, oa, ob) in &ors {
            if **a == ***oa && **b == ***ob {
                return Some((**or_pos, **a_start, **a_end, **b_start, **b_end));
            }
        }
    }

    None
}
fn find_symbol_in_and<T: Hash + Eq + PartialEq>(
    nk: &Logic<T>,
    state: &[State<T>],
) -> Option<(bool, usize)> {
    for s in state.iter().rev() {
        for ((k, o), (p, _)) in s.symbols.iter() {
            if o.is_some() {
                continue;
            }
            match &k {
                Logic::And(lk, _) if *nk == **lk => return Some((true, *p)),
                Logic::And(_, lk) if *nk == **lk => return Some((false, *p)),
                _ => continue,
            }
        }
    }
    None
}
#[allow(clippy::type_complexity)]
fn find_symbol<'a, T: Hash + Eq + PartialEq>(
    n: &(Logic<T>, Option<Logic<T>>),
    state: &'a [State<T>],
) -> Option<(&'a (Logic<T>, Option<Logic<T>>), (usize, usize))> {
    for s in state.iter().rev() {
        if let Some((r, p)) = s.symbols.get_key_value(n) {
            return Some((r, *p));
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
    ) -> Result<(Option<(Logic<T>, usize)>, Option<(Logic<T>, usize)>), String> {
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
                    if let Some(((f, fp), (l, lp))) = f.and_then(|f| l.map(|l| (f, l))) {
                        let last = stack.last_mut().unwrap();
                        last.symbols.insert((f, Some(l)), (fp, lp));
                    }
                }
                Line::Log(l, t) => {
                    *index += 1;
                    // println!("Checking: {}\t| State: {stack:?}", l.display(true));
                    if first.is_none() {
                        first = Some((l.as_ref().clone(), *index));
                    }
                    if last.is_none() && i + 1 == proof_len {
                        last = Some((l.as_ref().clone(), *index));
                    }

                    if is_first && stack.last_mut().unwrap().can_assume {
                        *t = Some(Instruction::Assumption);
                        stack
                            .last_mut()
                            .unwrap()
                            .symbols
                            .insert(((**l).clone(), None), (*index, 0));
                        continue;
                    }

                    // not not elim
                    if matches!(**l, Logic::Empty) {
                        *t = Some(Instruction::Invalid);
                    } else if let Some(((_, _), (index, _))) =
                        find_symbol(&(Logic::Not(Logic::Not(l.clone()).into()), None), &stack)
                    {
                        *t = Some(Instruction::NotNotElim(index));
                    }
                    // copy
                    else if let Some((_, (index, _))) = find_symbol(&(*l.clone(), None), &stack) {
                        *t = Some(Instruction::Copy(index));
                    }
                    // Impl elim
                    else if let Some((a, b)) = find_impl_elim(&*l, &stack) {
                        *t = Some(Instruction::ImplElim(a, b));
                    }
                    // Bottom elim
                    else if let Some((_, (index, _))) =
                        find_symbol(&(Logic::Bottom, None), &stack)
                    {
                        *t = Some(Instruction::BottomElim(index));
                    }
                    // and elim
                    else if let Some((left, index)) = find_symbol_in_and(&*l, &stack) {
                        if left {
                            *t = Some(Instruction::AndElimLeft(index))
                        } else {
                            *t = Some(Instruction::AndElimRight(index))
                        }
                    }
                    // or elim
                    else if let Some((o, a_start, a_end, b_start, b_end)) =
                        find_or_elim(&*l.clone(), &stack)
                    {
                        *t = Some(Instruction::OrElim(o, a_start..=a_end, b_start..=b_end))
                    }
                    // PBC
                    else if let Some((_, (a, b))) =
                        find_symbol(&(Logic::Not(l.clone()), Some(Logic::Bottom)), &stack)
                    {
                        *t = Some(Instruction::Pbc(a..=b));
                    } else {
                        match &mut **l {
                            // Not Not intro
                            Logic::Not(a) if matches!(&**a, Logic::Not(_)) => {
                                let Logic::Not(a) = &**a else { unreachable!() };
                                if let Some((_, (index, _))) =
                                    find_symbol(&(*a.clone(), None), &stack)
                                {
                                    *t = Some(Instruction::NotNotIntro(index));
                                } else {
                                    *t = Some(Instruction::Invalid);
                                }
                            }
                            // Not intro
                            Logic::Not(a) => {
                                if let Some((_, (a_index, b_index))) =
                                    find_symbol(&((**a).clone(), Some(Logic::Bottom)), &stack)
                                {
                                    *t = Some(Instruction::NotIntro(a_index..=b_index));
                                } else {
                                    *t = Some(Instruction::Invalid);
                                }
                            }
                            // And intro
                            Logic::And(a, b) => {
                                if let Some((na, nb)) = find_symbol(&(*a.clone(), None), &stack)
                                    .and_then(|(_, (a, _))| {
                                        find_symbol(&(*b.clone(), None), &stack)
                                            .map(|(_, (b, _))| (a, b))
                                    })
                                {
                                    *t = Some(Instruction::AndIntro(na, nb));
                                }
                            }
                            // Impl introduction
                            Logic::Implies(a, b) => {
                                if let Some((_, (a, b))) =
                                    find_symbol(&((**a).clone(), Some((**b).clone())), &stack)
                                {
                                    *t = Some(Instruction::ImplIntro(a..=b));
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
                                        if let Some((_, (p, _))) =
                                            find_symbol(&((**a).clone(), None), &stack)
                                        {
                                            Instruction::OrIntroLeft(p)
                                        } else if let Some((_, (p, _))) =
                                            find_symbol(&((**b).clone(), None), &stack)
                                        {
                                            Instruction::OrIntroRight(p)
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
                                    for ((term, o), (p_term, _)) in s.symbols.iter() {
                                        if o.is_some() {
                                            continue;
                                        }
                                        if let Some((_, (n_term, _))) = find_symbol(
                                            &(Logic::Not(term.clone().into()), None),
                                            &stack,
                                        ) {
                                            *t = Some(Instruction::NotElim(*p_term, n_term));
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
                    stack
                        .last_mut()
                        .unwrap()
                        .symbols
                        .insert((*l.clone(), None), (*index, 0));
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
        let mut res = false;
        for l in &self.0 {
            match l {
                Line::Sub(s) => res = res || s.has_invalid(),
                Line::Log(_, t) => {
                    res = res
                        || t.as_ref()
                            .map(|a| matches!(*a, Instruction::Invalid))
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
            state.symbols.insert((l.clone(), None), (i + 1, 0));
        });

        self.proof
            .verify(&mut self.prepositions.len(), vec![state])?;
        let ok = !self.proof.has_invalid()
            && self
                .proof
                .0
                .last()
                .map(|l| match l {
                    Line::Sub(_) => false,
                    Line::Log(l, _) => **l == *self.result,
                })
                .unwrap_or_default();
        Ok(ok)
    }
}
