use dioxus::prelude::*;
use nx_ui::prelude::*;

#[component]
pub fn ScrollAreaDemo() -> Element {
    rsx! {

        Card {
            CardHeader {
                CardTitle { "Scroll Area", }
            }
            CardContent {
                class: "card-content flex",
                ScrollArea {
                    class: "h-40",
                    p {
r#"Nexus (formerly MusterHub) is a high-performance,
enterprise-grade ecosystem designed for comprehensive training management,
administration, and resource allocation within complex organizational hierarchies.
Nexus abandons traditional monolithic API servers in favor of a Contract-First WebAssembly (WASI)
Component Model. Running inside secure sandboxes via the Fermyon Spin runtime and leveraging
an event-driven Redpanda / Kafka backbone, Nexus achieves instantaneous scaling, sub-millisecond
cold starts, and absolute fault isolation."#
                    }
                }
            }
        }
    }
}
