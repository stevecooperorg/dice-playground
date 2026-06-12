//! Sanitized literate report HTML from the engine (trusted after `ammonia` in weave).

use leptos::prelude::*;

/// Renders engine-sanitized literate weave output below the editor.
#[component]
pub fn LiterateReportView(#[prop(into)] html: String) -> impl IntoView {
    view! {
        <section class="mt-4 rounded-lg border border-slate-700 bg-slate-950 p-4 text-sm literate-report">
            <h2 class="font-semibold text-slate-200 mb-3 font-sans">"Report"</h2>
            <div class="literate-report-body text-slate-200 font-sans leading-relaxed" inner_html=html />
        </section>
    }
}
