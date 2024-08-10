#[allow(unused_imports)]
#[macro_use]
extern crate log;

use chrono::{DateTime, Local};
use dioxus::prelude::*;
use std::fmt::Write;
mod logic;
use logic::{empty, FitchProof, Line, Logic, Ptr, SelectType, SubProof};
use util::Droppable;
mod util;

fn main() {
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
fn SubProofComp<T: 'static + PartialEq + std::fmt::Display + Clone>(
    sub_proof: SubProof<T>,
    mut index: usize,
    index_map: Vec<usize>,
    unselectable: bool,
) -> Element {
    let GlobalProof(mut proof) = use_context();
    let TermSelector(mut index_map_ref) = use_context();
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
                    let pos = proof.write().proof.recurse(&index_map, |s| {
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
                },
                "+"
            }
        }
    })
}

#[component]
fn Proof() -> Element {
    let GlobalProof(proof) = use_context();
    let StartTime(start_time) = use_context();
    let WonTime(won_time) = use_context();
    let InfoScreen(mut info_screen) = use_context();
    // let TermSelector(debug) = use_context();

    let mut elapsed = use_signal(|| {
        Local::now()
            .signed_duration_since(start_time)
            .to_std()
            .unwrap_or_default()
    });
    use_coroutine::<(), _, _>(move |_: UnboundedReceiver<_>| async move {
        loop {
            elapsed.set(
                Local::now()
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

    let large_bottom;
    let body = if *info_screen.read() {
        large_bottom = false;
        let v = env!("CARGO_PKG_VERSION");
        rsx! (
            div {
                class: "info-screen-title",
                h1 {
                    "logiko v{v}"
                }
            }
            div {
                class: "info-screen-text",
                p {
                    "A puzzle game where you create Fitch-style "
                    "natural deduction proofs under a time limit."
                }
                p {
                    "Made over the course of a week while I should have studied for my "
                    "Logic re-exam."
                }
                p {
                    "As it was made rather quickly bugs may exist and the game is somewhat "
                    "rough around the edges, and if you find anything that should be improved "
                    "please email me at "
                    a {href:"mailto:bugs@loafey.se", "bugs@loafey.se"}
                    " or at loafey on Discord."
                }
                p {
                    "Source code can be found over at "
                    a {
                        href: "https://github.com/loafey/logiko",
                        "github.com/loafey/logiko"
                    }
                    " ("
                    span {
                        "🦀"
                    }
                    ")"
                }
                p {
                    "I currently have the following on my todo list:"
                    pre {
                        style: "white-space: pre-wrap",
                        "{include_str!(\"../notes.md\")}"
                    }
                }
            }

            button {
                onclick: move |_| info_screen.set(false),
                "Close"
            }
        )
    } else if let Some(time) = &*won_time.read() {
        large_bottom = true;
        let stats = proof.read().stats();
        let win_script = format!(
            r#"navigator.clipboard.writeText("🧩 I completed Logiko#{} in {time}s 🧩\nI used {} lines, {} sub proofs and {} terms\n\nhttps://loafey.se/logiko/")"#,
            day_since_start(),
            stats.lines,
            stats.sub_proofs,
            stats.terms
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
            }
        }
    } else {
        large_bottom = true;
        let ErrorField(error_field) = use_context();
        let error = if let Some(ef) = &*error_field.read() {
            rsx!(pre {
                "Please report all errors to bugs@loafey.se, or loafey on Discord.\n"
                "{ef}"
            })
        } else {
            rsx!()
        };
        rsx! {
            div {
                class: "title",
                "Puzzle: {day_since_start()}, "
                span {"{elapsed.read().as_secs()}s"}
                // span {" {debug:?}"}
            }

            div {
                class: "result-line",
                {result}
            }

            div {
                class: "sub-proof-outer",
                {error}

                for (ind, l) in pres.into_iter().enumerate() {
                    div {
                        class: "term-line-container",
                        pre { class: "term-rule", style: "padding-left: 15px", "{ind + 1}:" }
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
        }
    };
    let class = if large_bottom {
        "app-container"
    } else {
        "app-container app-container-info"
    };
    rsx!(div { class, {body} })
}

macro_rules! update_term {
    ($check:expr, $index_map_ref:expr, $proof:expr, $exp:expr) => {
        move |_| {
            let c = $index_map_ref.read().as_ref().unwrap().clone();
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
fn Keyboard() -> Element {
    let TermSelector(mut index_map_ref) = use_context();
    let GlobalProof(mut proof) = use_context();
    let WonTime(mut won_time) = use_context();
    let ErrorField(mut error_field) = use_context();
    let StartTime(start_time) = use_context();
    let InfoScreen(mut info_screen) = use_context();

    let res = proof.write().proof.recurse(
        index_map_ref.read().as_ref()?,
        |_| SelectType::SubProof,
        |_| SelectType::Term,
    )?;
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
                        let c = index_map_ref.read().as_ref().unwrap().clone();
                        proof.write().proof.remove_line(&c);
                        *index_map_ref.write() = None;
                        check();
                    },
                    "⌫"
                }
                button {
                    onclick: move |_| {
                        let index_map = index_map_ref.read();
                        proof.write().proof.make_sub_proof(index_map.as_ref().unwrap());
                        let new_map = index_map.clone().map(|mut m| {
                            m.push(0);
                            m
                        });
                        drop(index_map);
                        *index_map_ref.write() = new_map;
                        check();
                    },
                    "↵"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::And(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "∧"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::Or(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "∨"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::Implies(
                        Logic::Empty.into(),
                        Logic::Empty.into(),
                    )),
                    "→"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::Not(
                        Logic::Empty.into(),
                    )),
                    "¬"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::Bottom),
                    "⊥"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::Variable("p")),
                    "p"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::Variable("q")),
                    "q"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::Variable("r")),
                    "r"
                }
                button {
                    onclick: update_term!(check, index_map_ref, proof, |l| *l = Logic::Variable("s")),
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

fn day_since_start() -> usize {
    let date_str = "Wed, 7 Aug 2024 10:52:37 +0200";
    let datetime = DateTime::parse_from_rfc2822(date_str).unwrap();
    Local::now().signed_duration_since(datetime).num_days() as usize
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ErrorField(Signal<Option<String>>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct GlobalProof(Signal<FitchProof<&'static str>>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct TermSelector(Signal<Option<Vec<usize>>>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct WonTime(Signal<Option<usize>>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartTime(DateTime<Local>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct InfoScreen(Signal<bool>);

#[allow(unused)]
fn app() -> Element {
    let GlobalProof(mut sig) = use_context_provider(|| GlobalProof(Signal::new(empty())));
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let data = include_str!("../data.json");
        let json = serde_json::from_str::<Vec<FitchProof<&str>>>(data).unwrap();
        // *sig.write() = json[0].clone();
        *sig.write() = json[day_since_start() % json.len()].clone();
    });
    use_context_provider(|| ErrorField(Signal::new(None)));
    use_context_provider(|| TermSelector(Signal::new(Some(vec![0]))));
    use_context_provider(|| StartTime(Local::now()));
    use_context_provider(|| InfoScreen(Signal::new(false)));
    use_context_provider(|| WonTime(Signal::new(None)));
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
