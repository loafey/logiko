#[allow(unused_imports)]
#[macro_use]
extern crate log;

use chrono::DateTime;
use dioxus::prelude::*;
mod logic;
use logic::{empty, FitchProof, Line, Logic, Ptr, SelectType, SubProof};
use util::Droppable;
mod util;

fn main() {
    // let data = std::fs::read_to_string("data.json").unwrap();
    // let data = serde_json::from_str::<Vec<FitchProof<&str>>>(&data).unwrap();
    // for proof in data {
    //     println!("{proof}");
    // }

    // let mut proof = example_proof();
    // println!(" -- Input: --");
    // println!("{proof}");
    // println!(
    // " -- Proof is valid? --\n{}\n -- Proof with assumed rules: --",
    // proof.verify()
    // );
    // println!("{proof}");

    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    launch(app);
}

#[component]
fn Term<T: 'static + PartialEq + std::fmt::Display + Clone>(
    term: Ptr<Logic<T>>,
    outer: bool,
    index: Vec<usize>,
    unselectable: bool,
    other: bool,
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

    if matches!(&*term, Logic::Variable(_)) || outer {
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
    let mut proof = use_context::<Signal<FitchProof<&'static str>>>();
    let mut index_map_ref = use_context::<Signal<Option<Vec<usize>>>>();
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
                let ind = format!("{index:>2}");
                let a = a.map(|s| format!("{s}")).unwrap_or_default();
                rsx! {
                    div {
                        class: "term-line-container",
                        pre { class: "term-rule", "{ind}:" }
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

    let remove_button = if !index_map.is_empty() {
        rsx!(button {
            onclick: {
                let index_map = index_map.clone();
                move |_| {
                    proof.write().proof.remove_line(&index_map);
                    *index_map_ref.write() = None;
                }
            },
            "⌫"
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
            button {
                onclick: move |_| {
                    let mut c = index_map.clone();
                    c.push(index);
                    proof.write().proof.recurse(&index_map, |s| {
                        s.0.push(
                            Line::Log(Logic::Empty.into(), None)
                        )
                        ;
                    }, |_|{});
                    *index_map_ref.write() = Some(c.clone());
                },
                "+"
            }
        }
    })
}

#[component]
fn Proof() -> Element {
    let proof = use_context::<Signal<FitchProof<&'static str>>>();
    let start_time = use_context::<DateTime<chrono::Local>>();
    let mut elapsed = use_signal(|| {
        chrono::Local::now()
            .signed_duration_since(start_time)
            .to_std()
            .unwrap_or_default()
    });
    use_coroutine::<(), _, _>(move |_: UnboundedReceiver<_>| async move {
        loop {
            elapsed.set(
                chrono::Local::now()
                    .signed_duration_since(start_time)
                    .to_std()
                    .unwrap_or_default(),
            );
            wasmtimer::tokio::sleep(std::time::Duration::from_secs(1)).await
        }
    });

    let own_proof = proof.read().proof.clone();
    let result = rsx!(Term {
        term: proof.read().result.clone(),
        outer: true,
        index: Vec::new(),
        unselectable: true,
        other: false,
    });
    let pres = proof.read().prepositions.clone();
    let pres_len = pres.len();
    rsx! (div {
        class: "app-container",

        div {
            class: "title",
            "Puzzle: {day_since_start()}, {elapsed.read().as_secs()}s"
        }

        div {
            class: "result-line",
            {result}
        }

        div {
            class: "sub-proof-outer",
            for (ind, l) in pres.into_iter().enumerate() {
                div {
                    class: "term-line-container",
                    pre { class: "term-rule", style: "padding-left: 20px", "{ind + 1}:" }
                    div {
                        class: "term-line",
                        Term { term: Box::new(l), outer: true, index: Vec::new(), unselectable: true, other: false }
                        div { class: "term-rule", "{logic::Instruction::Premise}" }
                    }
                }
            }
            SubProofComp {
                sub_proof: own_proof,
                index: pres_len,
                index_map: Vec::new(),
                unselectable: false,
            }
        }
        Keyboard {}
    })
}

macro_rules! update_term {
    ($index_map_ref:expr, $proof:expr, $exp:expr) => {
        move |_| {
            let c = $index_map_ref.read().as_ref().unwrap().clone();
            $proof.write().proof.recurse(&c, |_| {}, $exp).drop();
            *$index_map_ref.write() = Some(c);
        }
    };
}

#[component]
fn Keyboard() -> Element {
    let mut index_map_ref = use_context::<Signal<Option<Vec<usize>>>>();
    let mut proof = use_context::<Signal<FitchProof<&'static str>>>();

    let res = proof.write().proof.recurse(
        index_map_ref.read().as_ref()?,
        |_| SelectType::SubProof,
        |_| SelectType::Term,
    )?;
    let check = move |_: Event<MouseData>| {
        proof.write().verify();
    };
    match res {
        SelectType::Term => rsx! (div {
            class: "keyboard",
            div {
                class: "keyboard-inner",
                button {
                    onclick: move |_| {
                        let c = index_map_ref.read().as_ref().unwrap().clone();
                        proof.write().proof.remove_line(&c);
                    },
                    "⌫"
                }
                button {
                    onclick: move |_| {
                        let index_map = index_map_ref.read();
                        proof.write().proof.make_sub_proof(index_map.as_ref().unwrap());
                        drop(index_map);
                        *index_map_ref.write() = None;
                    },
                    "↵"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::And(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "∧"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::Or(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "∨"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::Implies(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "→"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::Not(
                        Logic::Empty.into(),
                    )),
                    "¬"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::Bottom),
                    "⊥"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::Variable("p")),
                    "p"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::Variable("q")),
                    "q"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::Variable("r")),
                    "r"
                }
                button {
                    onclick: update_term!(index_map_ref, proof, |l| *l = Logic::Variable("s")),
                    "s"
                }
                button { onclick: check, "🔎" }
            }
        }),
        SelectType::SubProof => rsx! { button { onclick: check, "🔎" }},
    }
}

fn day_since_start() -> usize {
    let date_str = "Wed, 7 Aug 2024 10:52:37 +0200";
    let datetime = DateTime::parse_from_rfc2822(date_str).unwrap();
    chrono::Local::now()
        .signed_duration_since(datetime)
        .num_days() as usize
}

#[allow(unused)]
fn app() -> Element {
    let mut sig = use_context_provider(|| Signal::new(empty()));
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let data = include_str!("../data.json");
        let json = serde_json::from_str::<Vec<FitchProof<&str>>>(data).unwrap();
        // *sig.write() = json[0].clone();
        *sig.write() = json[day_since_start() % json.len()].clone();
    });
    use_context_provider::<Signal<Option<Vec<usize>>>>(|| Signal::new(Some(vec![0])));
    use_context_provider(|| Signal::new(0usize));
    use_context_provider(chrono::Local::now);
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
