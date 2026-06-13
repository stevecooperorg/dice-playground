//! Report tab HTML with Chartistry mounted into engine placeholders.

use leptos::mount::mount_to;
use leptos::prelude::*;
use leptos::tachys::view::any_view::AnyViewState;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

use crate::engine::{output_entry_name, OutputEntry};

use super::output_graph::OutputEntryChart;

fn find_output_by_name(outputs: &[OutputEntry], name: &str) -> Option<OutputEntry> {
    outputs
        .iter()
        .rev()
        .find(|e| output_entry_name(e) == name)
        .cloned()
}

#[component]
pub fn ReportHtmlHost(
    #[prop(into)] html: Signal<String>,
    #[prop(into)] outputs: Signal<Vec<OutputEntry>>,
) -> impl IntoView {
    let host = NodeRef::<leptos::html::Div>::new();
    let chart_handles = Rc::new(RefCell::new(Vec::<leptos::mount::UnmountHandle<AnyViewState>>::new()));

    Effect::new({
        let chart_handles = chart_handles.clone();
        move |_| {
        let html = html.get();
        let outputs = outputs.get();
        let Some(host_el) = host.get() else {
            return;
        };
        chart_handles.borrow_mut().clear();
        let host_el: Element = host_el.into();
        host_el.set_inner_html(&html);

        let Ok(node_list) = host_el.query_selector_all(".dice-output-chart[data-dice-output]") else {
            return;
        };

        let mut new_handles = Vec::new();
        for i in 0..node_list.length() {
            let Some(node) = node_list.item(i) else {
                continue;
            };
            let Some(el) = node.dyn_ref::<Element>() else {
                continue;
            };
            let name = el.get_attribute("data-dice-output").unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let Some(entry) = find_output_by_name(&outputs, &name) else {
                continue;
            };
            let Ok(placeholder) = el.clone().dyn_into::<HtmlElement>() else {
                continue;
            };
            let handle = mount_to(placeholder, move || view! { <OutputEntryChart entry=entry.clone() /> });
            new_handles.push(handle);
        }
        *chart_handles.borrow_mut() = new_handles;
        }
    });

    view! {
        <div
            node_ref=host
            class="literate-report-body text-slate-200 font-sans leading-relaxed text-sm"
        />
    }
}
