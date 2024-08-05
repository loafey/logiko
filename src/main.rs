use std::rc::Rc;

use dioxus::prelude::*;
mod logic;
use logic::{example_proof, FitchProof, Line, Logic, Ptr, SubProof};

fn main() {
    launch(app);
}

#[component]
fn Term<T: 'static + PartialEq + std::fmt::Display + Clone>(
    term: Ptr<Logic<T>>,
    outer: bool,
    index: Vec<usize>,
) -> Element {
    let mut index_var = use_context::<Signal<Vec<usize>>>();
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

    let term_gui = match &*term.borrow() {
        Logic::Variable(v) => rsx!("{v}"),
        Logic::And(a, b) => rsx!(
            Term {term: a.clone(), outer: false, index: index0}
            " ∧ "
            Term {term: b.clone(), outer: false, index: index1}
        ),
        Logic::Or(a, b) => rsx!(
            Term {term: a.clone(), outer: false, index: index0}
            " ∨ "
            Term {term: b.clone(), outer: false, index: index1}
        ),
        Logic::Implies(a, b) => rsx!(
            Term {term: a.clone(), outer: false, index: index0}
            " → "
            Term {term: b.clone(), outer: false, index: index1}
        ),
        Logic::Equivalent(_, _) => rsx!("Equivalent"),
        Logic::Not(t) if matches!(&*t.borrow(), Logic::Variable(_)) => {
            rsx!("¬ " Term { term: t.clone(), outer: true, index: index0 })
        }
        Logic::Not(t) => rsx!("¬ " Term { term: t.clone(), outer: false, index: index0 }),
        Logic::Bottom => rsx!("⊥"),
        Logic::Empty => rsx!("×"),
    };
    let on_click = move |e: Event<MouseData>| {
        e.stop_propagation();
        *index_var.write() = index.clone();
    };
    if matches!(&*term.borrow(), Logic::Variable(_)) || outer {
        rsx!(div {
            onclick: on_click,
            class: "term",
            {term_gui}
        })
    } else {
        rsx!(div {
            onclick: on_click,
            class: "term",
            "( " {term_gui} " )"
        })
    }
}

#[component]
fn SubProofComp<T: 'static + PartialEq + std::fmt::Display + Clone>(
    sub_proof: SubProof<T>,
    mut index: usize,
    index_map: Vec<usize>,
) -> Element {
    let SubProof(lines) = sub_proof;
    let lines = lines.into_iter().enumerate().map(|(i, line)| {
        let mut c = index_map.clone();
        c.push(i);

        match line {
            Line::Sub(s) => {
                let l = s.len();
                let res = rsx!(SubProofComp {
                    sub_proof: s,
                    index: index,
                    index_map: c
                });
                index += l;
                res
            }
            Line::Log(l, a) => {
                index += 1;
                rsx! {
                    div {
                        class: "term-line-container",
                        div { class: "term-rule", "{index}:" }
                        div {
                            class: "term-line",
                            Term { term: l, outer: true, index: c }
                            div { class: "term-rule", "{a}" }
                        }
                    }
                }
            }
        }
    });

    rsx!(div {
        class: "sub-proof",
        for line in lines {
            {line}
        }
    })
}

#[component]
fn Proof() -> Element {
    let proof = use_context::<Signal<FitchProof<&'static str>>>();
    let own_proof = proof.read().proof.clone();
    let result = rsx!(Term {
        term: proof.read().result.clone(),
        outer: true,
        index: Vec::new()
    });
    let debug = use_context::<Signal<Vec<usize>>>();

    rsx! (div {
        class: "app-container",
        SubProofComp {
            sub_proof: own_proof,
            index: 0,
            index_map: Vec::new()
        }
        "Goal: {debug:?}"
        div {
            class: "result-line",
            div { class: "term-rule", "⊢ " }
            {result}
        }
    })
}

fn app() -> Element {
    use_context_provider(|| Signal::new(example_proof()));
    use_context_provider::<Signal<Vec<usize>>>(|| Signal::new(vec![]));
    use_context_provider(|| Signal::new(0usize));
    let style = grass::include!("src/style.scss");

    rsx! {
        style { "{style}" }
        div {
            class: "app-outer",
            Proof {}
        }
        // pre { "{proof}" }
    }
}
