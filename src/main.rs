use std::sync::Arc;

use dioxus::prelude::*;
mod logic;
use logic::{example_proof, FitchProof, Logic};

fn main() {
    launch(app);
}

#[component]
fn Term<T: 'static + PartialEq + std::fmt::Display + Clone>(
    term: Arc<Logic<T>>,
    outer: bool,
) -> Element {
    let term_gui = match &*term {
        Logic::Variable(v) => rsx!("{v}"),
        Logic::And(a, b) => rsx!(
            Term {term: a.clone(), outer: false}
            " ∧ "
            Term {term: b.clone(), outer: false}
        ),
        Logic::Or(a, b) => rsx!(
            Term {term: a.clone(), outer: false}
            " ∨ "
            Term {term: b.clone(), outer: false}
        ),
        Logic::Implies(a, b) => rsx!(
            Term {term: a.clone(), outer: false}
            " → "
            Term {term: b.clone(), outer: false}
        ),
        Logic::Equivalent(_, _) => rsx!("Equivalent"),
        Logic::Not(_) => rsx!("Not"),
        Logic::Bottom => rsx!("Bottom"),
    };
    if matches!(&*term, Logic::Variable(_)) || outer {
        rsx!(div {
            class: "term",
            {term_gui}
        })
    } else {
        rsx!(div {
            class: "term",
            "( " {term_gui} " )"
        })
    }
}

#[component]
fn Proof() -> Element {
    let proof = use_context::<Signal<FitchProof<&'static str>>>();
    let result = rsx!(Term {
        term: proof.read().result.clone(),
        outer: true
    });
    rsx! {

        div {
            class: "term-line",
            {result}
        }
    }
}

fn app() -> Element {
    let proof = use_context_provider(|| Signal::new(example_proof()));
    let style = grass::include!("src/style.scss");

    rsx! {
        style { "{style}" }
        Proof {}
        pre { "{proof}" }
    }
}
