//! Stacked textarea + highlight backdrop for `.dice` scripts.

use leptos::ev;
use leptos::html::{Div, Textarea};
use leptos::prelude::*;

use super::highlight::highlight_line;

pub const HIGHLIGHTED_EDITOR_STYLES: &str = r"
.highlighted-editor {
  position: relative;
  display: block;
  width: 100%;
}
.highlighted-editor-inner {
  position: relative;
  width: 100%;
  min-height: inherit;
}
.highlighted-editor-backdrop {
  position: absolute;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  white-space: pre-wrap;
  overflow-wrap: break-word;
  word-break: normal;
  margin: 0;
  border: none;
  padding: inherit;
  font: inherit;
  line-height: inherit;
  letter-spacing: inherit;
  tab-size: inherit;
  box-sizing: border-box;
}
.highlighted-editor-textarea {
  position: relative;
  z-index: 1;
  display: block;
  width: 100%;
  min-height: inherit;
  margin: 0;
  border: none;
  outline: none;
  resize: vertical;
  background: transparent;
  color: transparent;
  caret-color: #f1f5f9;
  font: inherit;
  line-height: inherit;
  letter-spacing: inherit;
  tab-size: inherit;
  padding: inherit;
  box-sizing: border-box;
  white-space: pre-wrap;
  overflow-wrap: break-word;
  word-break: normal;
  overflow: auto;
  scrollbar-gutter: stable;
}
.highlighted-editor-textarea::selection {
  background: rgba(56, 189, 248, 0.35);
}
.tok-plain, .tok-id { color: #f1f5f9; }
.tok-kw { color: #7dd3fc; }
.tok-str { color: #fcd34d; }
.tok-num { color: #93c5fd; }
.tok-com { color: #64748b; }
.tok-dice { color: #34d399; }
.tok-op { color: #94a3b8; }
";

#[component]
pub fn HighlightedEditor(
    value: Memo<String>,
    on_input: impl Fn(ev::Event) + Clone + Send + 'static,
    on_keydown: impl Fn(ev::KeyboardEvent) + Clone + Send + 'static,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let backdrop_ref = NodeRef::<Div>::new();
    let textarea_ref = NodeRef::<Textarea>::new();

    let highlighted_lines = Memo::new(move |_| {
        let text = value.get();
        text.split('\n').map(highlight_line).collect::<Vec<_>>()
    });

    let sync_scroll = move || {
        let Some(ta) = textarea_ref.get() else {
            return;
        };
        let Some(backdrop) = backdrop_ref.get() else {
            return;
        };
        backdrop.set_scroll_top(ta.scroll_top());
        backdrop.set_scroll_left(ta.scroll_left());
    };

    let on_scroll = move |_ev: ev::Event| {
        sync_scroll();
    };

    let on_input_handler = {
        let on_input = on_input.clone();
        move |ev: ev::Event| {
            on_input(ev);
            sync_scroll();
        }
    };

    let on_keydown_handler = {
        let on_keydown = on_keydown.clone();
        move |ev: ev::KeyboardEvent| on_keydown(ev)
    };

    view! {
        <style>{HIGHLIGHTED_EDITOR_STYLES}</style>
        <div class=format!("highlighted-editor {class}")>
            <div class="highlighted-editor-inner">
                <div
                    node_ref=backdrop_ref
                    class="highlighted-editor-backdrop"
                    aria-hidden="true"
                >
                    {move || {
                        let lines = highlighted_lines.get();
                        let last_idx = lines.len().saturating_sub(1);
                        lines
                            .into_iter()
                            .enumerate()
                            .map(|(i, line_spans)| {
                                view! {
                                    <>
                                        {line_spans
                                            .into_iter()
                                            .map(|span| {
                                                let class = span.class_name().to_string();
                                                view! {
                                                    <span class=class>{span.text}</span>
                                                }
                                            })
                                            .collect_view()}
                                        {(i < last_idx).then(|| view! { <br/> })}
                                    </>
                                }
                            })
                            .collect_view()
                    }}
                </div>
                <textarea
                    node_ref=textarea_ref
                    class="highlighted-editor-textarea"
                    prop:value=move || value.get()
                    spellcheck="false"
                    autocomplete="off"
                    on:input=on_input_handler
                    on:keydown=on_keydown_handler
                    on:scroll=on_scroll
                />
            </div>
        </div>
    }
}
