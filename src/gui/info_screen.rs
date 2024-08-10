use super::super::InfoScreen;
use dioxus::prelude::*;

#[component]
pub fn GuiInfoScreen() -> Element {
    let InfoScreen(mut info_screen) = use_context();
    let v = env!("CARGO_PKG_VERSION");
    rsx!(
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
                    "{include_str!(\"../../notes.md\")}"
                }
            }
        }

        button {
            onclick: move |_| info_screen.set(false),
            "Close"
        }
    )
}
