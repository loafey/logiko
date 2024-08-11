use crate::{
    logic::{Logic, SelectType},
    util::Droppable,
    ErrorField, GlobalProof, InfoScreen, StartTime, TermSelector, UndoStack, WonTime,
};
use chrono::Local;
use dioxus::prelude::*;
use std::fmt::Write;

macro_rules! update_term {
    ($undo_stack:expr, $check:expr, $index_map_ref:expr, $proof:expr, $exp:expr) => {
        move |_| {
            let c = $index_map_ref.read().as_ref().unwrap().clone();
            $undo_stack.push($proof.write().clone(), $index_map_ref.write().clone());
            $proof.write().proof.recurse(&c, |_| {}, $exp).drop();
            if let Some(new_index) = $proof.write().next_select(&c) {
                *$index_map_ref.write() = Some(new_index);
            } else {
                *$index_map_ref.write() = Some(c);
            }
            $check();
        }
    };
}

#[component]
pub fn Keyboard() -> Element {
    let TermSelector(mut index_map_ref) = use_context();
    let GlobalProof(mut proof) = use_context();
    let WonTime(mut won_time) = use_context();
    let ErrorField(mut error_field) = use_context();
    let StartTime(start_time) = use_context();
    let InfoScreen(mut info_screen) = use_context();
    let mut undo_stack = use_context::<UndoStack>();

    let can_undo = !undo_stack.0.read().is_empty();

    let (res, is_outer) = {
        let mut p = proof.write();
        let res = p.proof.recurse(
            index_map_ref.read().as_ref()?,
            |_| SelectType::SubProof,
            |_| SelectType::Term,
        )?;
        let outer = p.proof.is_outer_term(index_map_ref.read().as_ref()?);
        (res, outer)
    };

    let mut check = move || {
        match proof.write().verify() {
            Ok(b) => {
                if b {
                    *won_time.write() = Some(
                        Local::now()
                            .signed_duration_since(start_time)
                            .to_std()
                            .unwrap_or_default()
                            .as_secs() as usize,
                    );
                }
            }
            Err(s) => {
                let mut r = error_field.write();
                if let Some(field) = &mut *r {
                    let _ = write!(field, "\n{s}");
                } else {
                    *r = Some(s);
                }
            }
        };
    };
    match res {
        SelectType::Term => rsx! (div {
            class: "keyboard",
            div {
                class: "keyboard-inner",
                button {
                    onclick: move |_| {
                        let pop = undo_stack.undo();
                        if let Some((pop,index)) = pop {
                            *proof.write() = pop;
                            *index_map_ref.write() = index;
                        }
                    },
                    disabled: !can_undo,
                    "⎌"
                }
                button {
                    onclick: move |_| {
                        let c = index_map_ref.read().as_ref().unwrap().clone();
                        undo_stack.push(proof.write().clone(), Some(c.clone()));
                        proof.write().proof.remove_line(&c);
                        *index_map_ref.write() = None;
                        check();
                    },
                    disabled: !is_outer,
                    "⌫"
                }
                button {
                    onclick: move |_| {
                        let index_map = index_map_ref.read();
                        undo_stack.push(proof.write().clone(), index_map.clone());
                        proof.write().proof.make_sub_proof(index_map.as_ref().unwrap());
                        let new_map = index_map.clone().map(|mut m| {
                            m.push(0);
                            m
                        });
                        drop(index_map);
                        *index_map_ref.write() = new_map;
                        check();
                    },
                    disabled: !is_outer,
                    "↵"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::And(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "∧"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::Or(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "∨"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::Implies(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "→"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::Not(
                        Logic::Empty.into(),
                    )),
                    "¬"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::Bottom),
                    "⊥"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::Variable("p")),
                    "p"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::Variable("q")),
                    "q"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::Variable("r")),
                    "r"
                }
                button {
                    onclick: update_term!(undo_stack, check, index_map_ref, proof, |l| *l = Logic::Variable("s")),
                    "s"
                }
                button {
                    onclick: move |_| {
                        info_screen.set(true);
                    },
                    "?"
                }
            }
        }),
        SelectType::SubProof => rsx! { button { onclick: move |_| check(), "🔎" }},
    }
}
