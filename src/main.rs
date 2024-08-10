#[allow(unused_imports)]
#[macro_use]
extern crate log;

use chrono::{DateTime, Local};
use dioxus::prelude::*;
use gui::{GuiInfoScreen, Keyboard, SubProofComp, Term};
mod logic;
use logic::{empty, FitchProof};
mod gui;
mod util;

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    launch(app);
}

#[component]
fn MainApp() -> Element {
    let GlobalProof(proof) = use_context();
    let StartTime(start_time) = use_context();
    let WonTime(won_time) = use_context();
    let InfoScreen(info_screen) = use_context();
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
        rsx!(GuiInfoScreen {})
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
    use_context_provider(|| InfoScreen(Signal::new(true)));
    use_context_provider(|| WonTime(Signal::new(None)));
    let style = grass::include!("src/style.scss");

    rsx! {
        style { "{style}" }
        div {
            class: "app-outer",
            MainApp {}
        }
        // pre { "{proof}" }
    }
}
