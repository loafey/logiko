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
    unselectable: bool,
) -> Element {
    let mut index_var = use_context::<Signal<Option<Vec<usize>>>>();
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
            Term {term: a.clone(), outer: false, index: index0, unselectable}
            " ∧ "
            Term {term: b.clone(), outer: false, index: index1, unselectable}
        ),
        Logic::Or(a, b) => rsx!(
            Term {term: a.clone(), outer: false, index: index0, unselectable}
            " ∨ "
            Term {term: b.clone(), outer: false, index: index1, unselectable}
        ),
        Logic::Implies(a, b) => rsx!(
            Term {term: a.clone(), outer: false, index: index0, unselectable}
            " → "
            Term {term: b.clone(), outer: false, index: index1, unselectable}
        ),
        Logic::Equivalent(_, _) => rsx!("Equivalent"),
        Logic::Not(t) if matches!(&*t.borrow(), Logic::Variable(_)) => {
            rsx!("¬ " Term { term: t.clone(), outer: true, index: index0, unselectable })
        }
        Logic::Not(t) => {
            rsx!("¬ " Term { term: t.clone(), outer: false, index: index0, unselectable })
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

    if matches!(&*term.borrow(), Logic::Variable(_)) || outer {
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
fn SubProofComp<T: 'static + PartialEq + std::fmt::Display + Clone>(
    sub_proof: SubProof<T>,
    mut index: usize,
    index_map: Vec<usize>,
    unselectable: bool,
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
                    index_map: c,
                    unselectable
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
                            Term { term: l, outer: true, index: c, unselectable }
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
        index: Vec::new(),
        unselectable: true,
    });
    let debug = use_context::<Signal<Option<Vec<usize>>>>();

    rsx! (div {
        class: "app-container",
        "debug index: {debug:?}"
        SubProofComp {
            sub_proof: own_proof,
            index: 0,
            index_map: Vec::new(),
            unselectable: false
        }
        "Goal:"
        div {
            class: "result-line",
            div { class: "term-rule", "⊢ " }
            {result}
        }
    })
}

fn app() -> Element {
    use_context_provider(|| Signal::new(example_proof()));
    use_context_provider::<Signal<Option<Vec<usize>>>>(|| Signal::new(None));
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
