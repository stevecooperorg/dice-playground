//! Tabbed playground output: Report (HTML), text, json, graph.

use leptos::prelude::*;

use crate::engine::OutputEntry;

use super::output_graph::OutputGraphView;
use super::report_html_host::ReportHtmlHost;

const TAB_HTML: &str = "html";
const TAB_TEXT: &str = "text";
const TAB_JSON: &str = "json";
const TAB_GRAPH: &str = "graph";

fn tab_button_class(active: bool) -> &'static str {
    if active {
        "text-xs px-2 py-1 rounded bg-slate-700"
    } else {
        "text-xs px-2 py-1 rounded hover:bg-slate-800"
    }
}

/// Combined Report tab HTML (full literate weave or legacy outputs-only).
#[component]
pub fn OutputPanelView(
    #[prop(into)] tab: Signal<String>,
    set_tab: WriteSignal<String>,
    #[prop(into)] report_html: Signal<String>,
    #[prop(into)] text: Signal<String>,
    #[prop(into)] json: Signal<String>,
    #[prop(into)] outputs: Signal<Vec<OutputEntry>>,
) -> impl IntoView {
    let show_report = move || !report_html.get().is_empty();

    view! {
        <section class="mt-4 rounded-lg border border-slate-700 bg-slate-950 p-3 text-sm font-mono">
            <div class="flex items-center gap-2 mb-2 not-font-mono font-sans flex-wrap" role="tablist">
                <h2 class="font-semibold text-slate-200 mr-1">"Output"</h2>
                {move || {
                    show_report().then(|| {
                        view! {
                            <button
                                type="button"
                                role="tab"
                                aria-selected=move || tab.get() == TAB_HTML
                                class=move || tab_button_class(tab.get() == TAB_HTML)
                                on:click=move |_| set_tab.set(TAB_HTML.to_owned())
                            >
                                "report"
                            </button>
                        }
                    })
                }}
                <button
                    type="button"
                    role="tab"
                    aria-selected=move || tab.get() == TAB_TEXT
                    class=move || tab_button_class(tab.get() == TAB_TEXT)
                    on:click=move |_| set_tab.set(TAB_TEXT.to_owned())
                >
                    "text"
                </button>
                <button
                    type="button"
                    role="tab"
                    aria-selected=move || tab.get() == TAB_JSON
                    class=move || tab_button_class(tab.get() == TAB_JSON)
                    on:click=move |_| set_tab.set(TAB_JSON.to_owned())
                >
                    "json"
                </button>
                <button
                    type="button"
                    role="tab"
                    aria-selected=move || tab.get() == TAB_GRAPH
                    class=move || tab_button_class(tab.get() == TAB_GRAPH)
                    on:click=move |_| set_tab.set(TAB_GRAPH.to_owned())
                >
                    "graph"
                </button>
            </div>
            <div role="tabpanel" class="not-font-mono font-sans">
                {move || {
                    let current = tab.get();
                    if current == TAB_HTML && show_report() {
                        view! {
                            <ReportHtmlHost html=report_html outputs=outputs />
                        }
                        .into_any()
                    } else if current == TAB_GRAPH {
                        view! {
                            <div class="w-full min-w-0">
                                <OutputGraphView outputs=outputs.get() />
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <pre class="whitespace-pre-wrap break-words m-0 font-mono text-sm text-slate-200">
                                {if current == TAB_JSON {
                                    json.get()
                                } else {
                                    text.get()
                                }}
                            </pre>
                        }
                        .into_any()
                    }
                }}
            </div>
        </section>
    }
}

/// Pick initial tab after a successful Run.
pub fn default_output_tab(report_html: &str, text: &str) -> &'static str {
    if !report_html.is_empty() {
        TAB_HTML
    } else if !text.is_empty() {
        TAB_TEXT
    } else {
        TAB_JSON
    }
}
