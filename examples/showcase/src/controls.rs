use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub(crate) fn ControlsDemo() -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "Controls", }
            }
            CardContent {
                div {
                    class: "flex flex-col gap-4",
                    div {
                        class: "flex gap-5",
                        Checkbox { "Default" }
                        Checkbox { checked: CheckboxState::Checked, "Checked" }
                        Checkbox { checked: CheckboxState::Indeterminate, "Indeterminate" }
                        Checkbox { disabled: true, "Disabled" }
                    }
                    div {
                        class: "flex gap-5",
                        Switch { "Default" }
                        Switch { checked: true, "Enabled" }
                        Switch { disabled: true, "Disabled" }
                    }
                    div {
                        class: "flex gap-5",
                        RadioGroup {
                            default_value: "option2",
                            RadioItem { value: "option1", index: 0usize, "Option 1" }
                            RadioItem { value: "option2", index: 1usize, "Option 2" }
                            RadioItem { value: "option3", index: 2usize, disabled: true, "Option 3" }
                        }
                        Select::<String> {
                            on_value_change: move |_v: Option<String>| {},

                            SelectTrigger {
                                SelectValue {}
                            }

                            SelectList {
                                SelectOption::<String> {
                                    value: "rust",
                                    index: 0usize,
                                    text_value: "Rust Language",
                                    "Rust Language"
                                }
                                SelectOption::<String> {
                                    value: "cpp",
                                    index: 1usize,
                                    text_value: "C++ Legacy",
                                    "C++ Legacy"
                                }
                                SelectOption::<String> {
                                    value: "ts",
                                    index: 2usize,
                                    text_value: "TypeScript",
                                    "TypeScript"
                                }
                                SelectOption::<String> {
                                    value: "js",
                                    index: 3usize,
                                    text_value: "JavaScript",
                                    "JavaScript"
                                }
                                SelectOption::<String> {
                                    value: "java",
                                    index: 4usize,
                                    text_value: "Java",
                                    "Java"
                                }
                            }
                        }
                    }

                    div {
                        class: "flex gap-5",
                        Toggle { b { "B" } }
                        ToggleGroup {
                            horizontal: true,
                            allow_multiple_pressed: true,
                            ToggleItem { index: 0usize, b { "B" } }
                            ToggleItem { index: 1usize, i { "I" } }
                            ToggleItem { index: 2usize, u { "U" } }
                        }
                    }
                }
            }
        }
    }
}
