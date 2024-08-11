use super::super::InfoScreen;
use dioxus::prelude::*;

#[component]
fn Rule(rule: &'static str, children: Element) -> Element {
    rsx!(div {
        class: "tutorial-rule-container",
        h2 {
            "{rule}"
        }
        div {
            class:"tutorial-rule-content",
            {children}
        }
    })
}

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

            div {
                class: "info-screen-title",
                h1 {
                    "Rules"
                }
            }
            p {
                "Here are all the rules that can be used in the game.
                If you have everything above the horizontal line, you
                can create the value below the line. 
                Vertical lines represents sub-proofs, and two rules side-by-side
                means that you can use either one."
            }
            p {
                "These examples use p,q and s as variables, but you can
                substitute them for any term."
            }
            // Pbc(RangeInclusive<usize>),                                  // Implemented
            // Lem,                                                         // Implemented
            div {
                class: "info-screen-grid",
                Rule {
                    rule: "And Introduction",
                    div { "p" }
                    div { "q" }
                    div {
                        class: "result",
                        "p ∧ q"
                    }
                }
                Rule {
                    rule: "And Elimination",
                    div {
                        class: "info-column",
                        div {
                            div { "p ∧ q" }
                            div {
                                class: "result",
                                "p"
                            }
                        }
                        div {
                            div { "p ∧ q" }
                            div {
                                class: "result",
                                "q"
                            }
                        }
                    }
                }
                Rule {
                    rule: "Or Introduction",
                    div {
                        class: "info-column",
                        div {
                            div { "p" }
                            div {
                                class: "result",
                                "p ∨ q"
                            }
                        }
                        div {
                            div { "q" }
                            div {
                                class: "result",
                                "p ∨ q"
                            }
                        }
                    }
                }
                Rule {
                    rule: "Or Elimination",
                    div { "p ∨ q" }
                    div {
                        class: "info-sub",
                        div { "p" }
                        div { "..." }
                        div { "s" }
                    }
                    div {
                        class: "info-sub",
                        div { "q" }
                        div { "..." }
                        div { "s" }
                    }
                    div {
                        class: "result",
                        "s"
                    }
                }
                Rule {
                    rule: "Implication Introduction",
                    div {
                        class: "info-sub",
                        div { "p" }
                        div { "..." }
                        div { "q" }
                    }
                    div {
                        class: "result",
                        "p → q"
                    }
                }
                Rule {
                    rule: "Implication Elimination",
                    div{ "p → q" }
                    div{ "p" }
                    div {
                        class: "result",
                        "q"
                    }
                }
                Rule {
                    rule: "Not Introduction",
                    div {
                        class: "info-sub",
                        div { "q" }
                        div { "..." }
                        div { "⊥" }
                    }
                    div {
                        class: "result",
                        "¬q"
                    }
                }
                Rule {
                    rule: "Not Elimination",
                    div { "q" }
                    div { "¬q" }
                    div {
                        class: "result",
                        "⊥"
                    }
                }
                Rule {
                    rule: "Not Not Introduction",
                    div { "q" }
                    div {
                        class: "result",
                        "¬¬q"
                    }
                }
                Rule {
                    rule: "Not Not Elimination",
                    div { "¬¬q" }
                    div {
                        class: "result",
                        "q"
                    }
                }
                Rule {
                    rule: "Bottom Elimination",
                    div { "⊥" }
                    div { class: "result", "q" }
                }
                Rule {
                    rule: "PBC",
                    div {
                        class: "info-sub",
                        div { "¬q" }
                        div { "..." }
                        div { "⊥" }
                    }
                    div {
                        class: "result",
                        "q"
                    }
                }
                Rule {
                    rule: "LEM",
                    div { class: "empty" }
                    div {
                        class: "result",
                        "¬q ∨ q"
                    }
                }
                Rule {
                    rule: "Copy",
                    div { "q" }
                    div {
                        class: "result",
                        "q"
                    }
                }
            }
        }

        button {
            onclick: move |_| info_screen.set(false),
            "Close"
        }
    )
}
