use crate::{GlobalProof, TermSelector, UndoStack};
use dioxus::prelude::*;
use logic_check::{Line, Logic, Ptr, SubProof};

#[component]
pub fn Term<T: 'static + PartialEq + std::fmt::Display + Clone>(
    term: Ptr<Logic<T>>,
    outer: bool,
    index: Vec<usize>,
    unselectable: bool,
    other: bool,
) -> Element {
    let TermSelector(mut index_var) = use_context();

    let class = if let Some(map) = index_var.read().as_ref() {
        map == &index
    } else {
        false
    };
    let class = if class && !unselectable {
        "term term-selected"
    } else {
        "term"
    };
    let class = format!("{class} term-repeat-{}", if other { 1 } else { 2 });

    let index0 = {
        let mut c = index.clone();
        c.push(0);
        c
    };
    let index1 = {
        let mut c = index.clone();
        c.push(1);
        c
    };

    let term_gui = match &*term {
        Logic::Variable(v) => rsx!("{v}"),
        Logic::And(a, b) => rsx!(
            Term {term: a.clone(), outer: false, index: index0, unselectable, other: !other}
            " ∧ "
            Term {term: b.clone(), outer: false, index: index1, unselectable, other: !other}
        ),
        Logic::Or(a, b) => rsx!(
            Term {term: a.clone(), outer: false, index: index0, unselectable, other: !other}
            " ∨ "
            Term {term: b.clone(), outer: false, index: index1, unselectable, other: !other}
        ),
        Logic::Implies(a, b) => rsx!(
            Term {term: a.clone(), outer: false, index: index0, unselectable, other: !other}
            " → "
            Term {term: b.clone(), outer: false, index: index1, unselectable, other: !other}
        ),
        Logic::Not(t) if matches!(&**t, Logic::Variable(_) | Logic::Not(_) | Logic::Empty) => {
            rsx!("¬ " Term { term: t.clone(), outer: true, index: index0, unselectable, other: !other })
        }
        Logic::Not(t) => {
            rsx!("¬ " Term { term: t.clone(), outer: false, index: index0, unselectable, other: !other })
        }
        Logic::Bottom => rsx!("⊥"),
        Logic::Empty => rsx!("×"),
    };
    let on_click = {
        move |e: Event<MouseData>| {
            e.stop_propagation();
            if !unselectable {
                *index_var.write() = Some(index.clone());
            }
        }
    };

    if matches!(
        &*term,
        Logic::Variable(_) | Logic::Not(_) | Logic::Empty | Logic::Bottom
    ) || outer
    {
        rsx!(div {
            onclick: on_click,
            class: class,
            {term_gui}
        })
    } else {
        rsx!(div {
            onclick: on_click,
            class: class,
            "( " {term_gui} " )"
        })
    }
}

#[component]
pub fn SubProofComp<T: 'static + PartialEq + std::fmt::Display + Clone>(
    sub_proof: SubProof<T>,
    mut index: usize,
    index_map: Vec<usize>,
    unselectable: bool,
) -> Element {
    let GlobalProof(mut proof) = use_context();
    let TermSelector(mut index_map_ref) = use_context();
    let SubProof(lines) = sub_proof;
    let mut undo_stack = use_context::<UndoStack>();
    let lines = lines.into_iter().enumerate().map(|(i, line)| {
        let mut c = index_map.clone();
        c.push(i);

        match line {
            Line::Sub(s) => {
                let l = s.len();
                let res = rsx!(SubProofComp {
                    sub_proof: s,
                    index: index,
                    index_map: c,
                    unselectable
                });
                index += l;
                res
            }
            Line::Log(l, a) => {
                index += 1;
                let ind = format!("{index:>2}");
                let a = a.map(|s| format!("{s}")).unwrap_or_default();
                rsx! {
                    div {
                        class: "term-line-container",
                        pre { class: "term-line-number", "{ind}:" }
                        div {
                            class: "term-line",
                            Term { term: l, outer: true, index: c, unselectable, other: false }
                            div { class: "term-rule", "{a}" }
                        }
                    }
                }
            }
        }
    });

    let remove_button = if !index_map.is_empty() && !unselectable {
        rsx!(button {
            class: "remove-sub-proof-button",
            ondoubleclick: {
                let index_map = index_map.clone();
                move |_| {
                    undo_stack.push(proof.write().clone(), index_map_ref.write().clone());
                    proof.write().proof.remove_line(&index_map);
                    *index_map_ref.write() = None;
                }
            },
            "⌫"
        })
    } else {
        rsx!()
    };

    let add_button = if !unselectable {
        rsx!(button {
            onclick: {
                let mut c = index_map.clone();
                move |_| {
                    undo_stack.push(proof.write().clone(), index_map_ref.write().clone());
                    let pos = proof.write().proof.recurse(&c, |s| {
                        let pos = s.0.len();
                        s.0.push(
                            Line::Log(Logic::Empty.into(), None)
                        );
                        Some(pos)
                    }, |_| None);
                    if let Some(Some(pos)) = pos {
                        c.push(pos);
                    }
                    *index_map_ref.write() = Some(c.clone());
                }
            },
            "+"
        })
    } else {
        rsx!()
    };

    rsx!(div {
        class: "sub-proof",
        for line in lines {
            {line}
        }
        div {
            {remove_button}
            {add_button}
        }
    })
}
