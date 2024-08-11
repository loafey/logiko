use crate::{
    day_since_start,
    gui::{SubProofComp, Term},
    GlobalProof,
};
use dioxus::prelude::*;

#[component]
pub fn WinScreen(time: usize) -> Element {
    let GlobalProof(proof) = use_context();
    let own_proof = proof.read().proof.clone();
    let pres = proof.read().prepositions.clone();
    let pres_len = pres.len();

    let stats = proof.read().stats();
    let win_script = format!(
        r#"navigator.clipboard.writeText("🧩 I completed logiko#{} in {time}s 🧩\nI used {} lines, {} sub proofs and {} terms\n\nhttps://loafey.se/logiko/")"#,
        day_since_start(),
        stats.lines,
        stats.sub_proofs,
        stats.terms
    );
    let copy_text_tree = format!(r#"navigator.clipboard.writeText(`{}`)"#, proof.read());
    let copy_latex_tree = format!(
        r#"navigator.clipboard.writeText({:?})"#,
        proof.read().latex()
    );

    rsx! {
        div {
            class: "title",
            "Congrats!"
        }

        div {
            class: "title",
            "You won in: {time}s"
        }

        div {
            class: "result-container",
            button {
                onclick: move |_| {
                    eval(&win_script);
                },
                "Copy Result"
            }

            button {
                onclick: move |_| {
                    eval(&copy_text_tree);
                },
                "Copy Proof"
            }

            button {
                onclick: move |_| {
                    eval(&copy_latex_tree);
                },
                "Copy LaTeX Tree"
            }

            div {
                class: "sub-proof-outer",
                for (ind, l) in pres.into_iter().enumerate() {
                    div {
                        class: "term-line-container",
                        pre { class: "term-rule", style: "padding-left: 15px", "{ind + 1}:" }
                        div {
                            class: "term-line",
                            Term { term: Box::new(l), outer: true, index: Vec::new(), unselectable: true, other: false }
                            div { class: "term-rule", "{crate::logic::Instruction::Premise}" }
                        }
                    }
                }
                SubProofComp {
                    sub_proof: own_proof,
                    index: pres_len,
                    index_map: Vec::new(),
                    unselectable: true,
                }
            }
        }
    }
}
