use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub(crate) fn SheetDemo() -> Element {
    let mut open = use_signal(|| false);
    let mut side = use_signal(|| SheetSide::Right);

    let open_sheet = move |s: SheetSide| {
        move |_| {
            side.set(s);
            open.set(true);
        }
    };

    rsx! {
        Card {
            CardHeader {
                CardTitle { "Sheet", }
            }
            CardContent {
                Button { variant: ButtonVariant::Outline, onclick: open_sheet(SheetSide::Top), "Top" }
                Button { variant: ButtonVariant::Outline, onclick: open_sheet(SheetSide::Right), "Right" }
                Button { variant: ButtonVariant::Outline, onclick: open_sheet(SheetSide::Bottom), "Bottom" }
                Button { variant: ButtonVariant::Outline, onclick: open_sheet(SheetSide::Left), "Left" }
            }
        }

        Sheet { open: open(), on_open_change: move |v| open.set(v),
            SheetContent {
                side: side(),

                SheetClose {}

                SheetHeader {
                    SheetTitle { "Sheet Title" }
                    SheetDescription { "Sheet description goes here." }
                }

                div {
                    display: "grid",
                    flex: "1 1 0%",
                    grid_auto_rows: "min-content",
                    gap: "1.5rem",
                    padding: "0 1rem",
                    div { display: "grid", gap: "0.75rem",
                        Label { html_for: "sheet-demo-name", "Name" }
                        Input {
                            id: "sheet-demo-name",
                            initial_value: "Name",
                        }
                    }
                    div { display: "grid", gap: "0.75rem",
                        Label { html_for: "sheet-demo-username", "Username" }
                        Input {
                            id: "sheet-demo-username",
                            initial_value: "Username",
                        }
                    }
                }

                SheetFooter {
                    Button { onclick: move |_| open.set(false), "Save changes" }
                    Button { variant: ButtonVariant::Outline, onclick: move |_| open.set(false), "Cancel" }
                    // SheetClose {
                    //     as: |attributes| rsx! {
                    //         Button { variant: ButtonVariant::Outline, attributes, "Cancel" }
                    //     },
                    // }
                }
            }
        }
    }
}
