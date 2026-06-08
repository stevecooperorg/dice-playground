use leptos::ev;
use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Element, ScrollBehavior, ScrollIntoViewOptions, ScrollLogicalPosition};

use crate::engine::OutputEntry;
use crate::ui::eval_client;
use crate::ui::highlighted_editor::HighlightedEditor;
use crate::ui::models::{DiceFile, UiDiagnostic, WorkspaceState};
use crate::ui::output_graph::OutputGraphView;
use crate::ui::storage::{load_workspace, save_workspace};

const DOC_URL: &str = "/docs/";
const GITHUB_URL: &str = "https://github.com/stevecooperorg/dice-playground";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScrollAfterRun {
    Output,
    Diagnostics,
}

fn scroll_into_view_smooth(el: &Element) {
    let opts = ScrollIntoViewOptions::new();
    opts.set_behavior(ScrollBehavior::Smooth);
    opts.set_block(ScrollLogicalPosition::Start);
    let _ = el.scroll_into_view_with_scroll_into_view_options(&opts);
}

#[component]
pub fn App() -> impl IntoView {
    let (workspace, set_workspace) = signal(load_workspace());
    let (diagnostics, set_diagnostics) = signal(Vec::<UiDiagnostic>::new());
    let (result_text, set_result_text) = signal(String::new());
    let (result_json, set_result_json) = signal(String::new());
    let (result_outputs, set_result_outputs) = signal(Vec::<OutputEntry>::new());
    let (output_tab, set_output_tab) = signal("text".to_string());
    let (error_banner, set_error_banner) = signal(String::new());
    let (check_token, set_check_token) = signal(0u64);
    let (menu_open, set_menu_open) = signal(false);
    let (files_open, set_files_open) = signal(false);
    let (scroll_after_run, set_scroll_after_run) = signal(None::<ScrollAfterRun>);
    let diagnostics_ref = NodeRef::<Div>::new();
    let output_ref = NodeRef::<Div>::new();
    let bump_check = move || set_check_token.update(|t| *t += 1);

    let persist = move |ws: WorkspaceState| {
        set_workspace.set(ws);
        if let Err(e) = save_workspace(&workspace.get_untracked()) {
            set_error_banner.set(format!("save failed: {e:?}"));
        }
    };

    let active_content = Memo::new(move |_| {
        workspace
            .get()
            .active_file()
            .map(|f| f.content.clone())
            .unwrap_or_default()
    });

    let active_name = Memo::new(move |_| {
        workspace
            .get()
            .active_file()
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "untitled.dice".to_owned())
    });

    let has_diagnostics = move || !error_banner.get().is_empty() || !diagnostics.get().is_empty();

    let has_output = move || !result_text.get().is_empty() || !result_json.get().is_empty();

    Effect::new(move |_| {
        let target = scroll_after_run.get();
        let _ = result_text.get();
        let _ = error_banner.get();
        let _ = diagnostics.get();
        let Some(target) = target else {
            return;
        };
        let diagnostics_ref = diagnostics_ref;
        let output_ref = output_ref;
        let set_scroll_after_run = set_scroll_after_run;
        spawn_local(async move {
            for delay in [0u32, 50, 150] {
                gloo_timers::future::TimeoutFuture::new(delay).await;
                let el = match target {
                    ScrollAfterRun::Output => output_ref.get(),
                    ScrollAfterRun::Diagnostics => diagnostics_ref.get(),
                };
                if let Some(el) = el {
                    scroll_into_view_smooth(&el);
                    set_scroll_after_run.set(None);
                    break;
                }
            }
            set_scroll_after_run.set(None);
        });
    });

    Effect::new(move |_| {
        let token = check_token.get();
        let check_token = check_token;
        let active_name = active_name;
        let active_content = active_content;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(400).await;
            if check_token.get_untracked() != token {
                return;
            }
            let path = active_name.get_untracked();
            let source = active_content.get_untracked();
            match eval_client::check_source(&path, &source).await {
                Ok(d) => set_diagnostics.set(d),
                Err(e) => set_error_banner.set(e),
            }
        });
    });

    let on_edit = move |ev: ev::Event| {
        let value = event_target_value(&ev);
        let mut ws = workspace.get_untracked();
        if let Some(f) = ws.active_file_mut() {
            f.content = value;
        }
        persist(ws);
        bump_check();
    };

    let run_script = move || {
        let path = active_name.get_untracked();
        let source = active_content.get_untracked();
        spawn_local(async move {
            match eval_client::eval_program(&path, &source, "decimal").await {
                Ok(r) => {
                    set_error_banner.set(String::new());
                    set_result_text.set(r.text);
                    set_result_json
                        .set(serde_json::to_string_pretty(&r.outputs).unwrap_or_default());
                    set_result_outputs.set(r.outputs);
                    set_scroll_after_run.set(Some(ScrollAfterRun::Output));
                }
                Err(e) => {
                    set_error_banner.set(e);
                    set_scroll_after_run.set(Some(ScrollAfterRun::Diagnostics));
                }
            }
        });
    };

    let new_file = move |_| {
        let mut ws = workspace.get_untracked();
        let name = ws.suggest_new_filename();
        let id = crate::ui::models::new_file_id();
        ws.files.push(DiceFile {
            id: id.clone(),
            name,
            content: String::new(),
        });
        ws.active_id = id;
        persist(ws);
        bump_check();
    };

    view! {
        <div class="min-h-screen bg-slate-900 text-slate-100">
            <header class="fixed top-0 left-0 right-0 z-50 flex flex-col gap-2 border-b border-slate-700/80 bg-slate-900 shadow-lg">
                <div class="flex items-center gap-2 px-3 py-2">
                    <div class="relative">
                        <button
                            type="button"
                            class="rounded-lg px-3 py-2 bg-slate-800 hover:bg-slate-700 text-sm font-medium"
                            aria-expanded=move || menu_open.get()
                            aria-haspopup="true"
                            on:click=move |_| set_menu_open.update(|o| *o = !*o)
                        >
                            "Menu"
                        </button>
                        {move || {
                            menu_open.get().then(|| view! {
                                <div
                                    class="absolute left-0 top-full mt-1 min-w-[10rem] rounded-lg border border-slate-600 bg-slate-800 py-1 shadow-xl"
                                    role="menu"
                                >
                                    <a
                                        href="/docs/"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        role="menuitem"
                                        class="block px-3 py-2 text-sm hover:bg-slate-700 border-b border-slate-600"
                                        on:click=move |_| set_menu_open.set(false)
                                    >
                                        "User guide"
                                    </a>
                                    <a
                                        href="/tutorial/"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        role="menuitem"
                                        class="block px-3 py-2 text-sm hover:bg-slate-700"
                                        on:click=move |_| set_menu_open.set(false)
                                    >
                                        "Tutorial"
                                    </a>
                                    <a
                                        href="/cookbook/"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        role="menuitem"
                                        class="block px-3 py-2 text-sm hover:bg-slate-700"
                                        on:click=move |_| set_menu_open.set(false)
                                    >
                                        "Cookbook"
                                    </a>
                                    <a
                                        href="/references/"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        role="menuitem"
                                        class="block px-3 py-2 text-sm hover:bg-slate-700"
                                        on:click=move |_| set_menu_open.set(false)
                                    >
                                        "Function reference"
                                    </a>
                                </div>
                            })
                        }}
                    </div>
                    <button
                        type="button"
                        class="rounded-lg w-9 h-9 flex items-center justify-center bg-slate-800 hover:bg-slate-700 text-lg leading-none font-medium"
                        title="New file"
                        aria-label="New file"
                        on:click=new_file
                    >
                        "+"
                    </button>
                    <span class="text-sm font-semibold text-slate-300 truncate">"Dice Playground"</span>
                    <span class="flex-1"></span>
                    <a
                        href=GITHUB_URL
                        target="_blank"
                        rel="noopener noreferrer"
                        class="rounded-lg w-9 h-9 flex items-center justify-center bg-slate-800 hover:bg-slate-700 text-slate-200"
                        title="Source on GitHub"
                        aria-label="Open GitHub repository in a new tab"
                    >
                        <svg
                            class="w-5 h-5"
                            viewBox="0 0 16 16"
                            fill="currentColor"
                            aria-hidden="true"
                        >
                            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.53-.49-.18-.46-.09-.99.1-1.23.09-.12.24-.39.42-.48C2.39 12.52 1.45 11.78 1.45 10.5c0-.87.31-1.58.83-2.14-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.52.56.83 1.27.83 2.14 0 1.28-.94 2.02-1.93 2.38.19.1.33.29.42.48.1.24.28.77.1 1.23 0 0-.52.86-2.53.49 0 .67.01 1.3.01 1.49 0 .21.15.45.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
                        </svg>
                    </a>
                    <a
                        href=DOC_URL
                        target="_blank"
                        rel="noopener noreferrer"
                        class="rounded-lg w-9 h-9 flex items-center justify-center bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold"
                        title="Documentation"
                        aria-label="Open documentation in a new tab"
                    >
                        "?"
                    </a>
                    <button
                        type="button"
                        class="rounded-lg px-4 py-2 bg-emerald-700 hover:bg-emerald-600 text-sm font-semibold"
                        title="Run (Shift+Enter in editor)"
                        on:click=move |_| run_script()
                    >
                        "Run"
                    </button>
                </div>

                <div class="flex items-center gap-2 px-3 pb-2 border-t border-slate-700/60">
                    <div class="relative shrink-0">
                        <button
                            type="button"
                            class="flex items-center gap-2 rounded-lg px-2.5 py-1.5 bg-slate-800 hover:bg-slate-700 text-sm max-w-[12rem]"
                            aria-expanded=move || files_open.get()
                            aria-haspopup="listbox"
                            title="Files"
                            on:click=move |_| set_files_open.update(|o| *o = !*o)
                        >
                            <svg
                                class="w-4 h-4 shrink-0 text-slate-300"
                                viewBox="0 0 24 24"
                                fill="currentColor"
                                aria-hidden="true"
                            >
                                <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                            </svg>
                            <span class="truncate text-slate-200">{move || active_name.get()}</span>
                        </button>
                        {move || {
                            files_open.get().then(|| view! {
                                <div
                                    class="absolute left-0 top-full mt-1 w-[min(20rem,calc(100vw-2rem))] max-h-64 overflow-y-auto rounded-lg border border-slate-600 bg-slate-800 py-1 shadow-xl"
                                    role="listbox"
                                >
                                    {move || {
                                        workspace.get().files.iter().map(|f| {
                                            let id = f.id.clone();
                                            let id_active = id.clone();
                                            let id_active_aria = id.clone();
                                            let id_select = id.clone();
                                            let id_delete = id.clone();
                                            let name = f.name.clone();
                                            view! {
                                                <div
                                                    class=move || {
                                                        if workspace.get().active_id == id_active {
                                                            "flex items-center gap-1 bg-slate-700/80"
                                                        } else {
                                                            "flex items-center gap-1 hover:bg-slate-700/50"
                                                        }
                                                    }
                                                    role="option"
                                                    aria-selected=move || workspace.get().active_id == id_active_aria
                                                >
                                                    <button
                                                        type="button"
                                                        class="flex-1 min-w-0 text-left px-3 py-2 text-sm truncate"
                                                        on:click=move |_| {
                                                            let mut ws = workspace.get_untracked();
                                                            ws.active_id = id_select.clone();
                                                            persist(ws);
                                                            bump_check();
                                                            set_files_open.set(false);
                                                        }
                                                    >
                                                        {name}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        class="shrink-0 px-2.5 py-2 text-slate-400 hover:text-red-300 hover:bg-slate-700 disabled:opacity-30"
                                                        title="Delete file"
                                                        aria-label="Delete file"
                                                        disabled=move || workspace.get().files.len() <= 1
                                                        on:click=move |ev| {
                                                            ev.stop_propagation();
                                                            let mut ws = workspace.get_untracked();
                                                            if ws.files.len() <= 1 {
                                                                return;
                                                            }
                                                            ws.files.retain(|file| file.id != id_delete);
                                                            if ws.active_id == id_delete {
                                                                ws.active_id = ws.files[0].id.clone();
                                                            }
                                                            persist(ws);
                                                            bump_check();
                                                        }
                                                    >
                                                        "✕"
                                                    </button>
                                                </div>
                                            }
                                        }).collect_view()
                                    }}
                                </div>
                            })
                        }}
                    </div>
                    <input
                        class="min-w-0 flex-1 bg-slate-800 px-3 py-1.5 rounded text-sm"
                        placeholder="Filename"
                        prop:value=active_name
                        on:input=move |ev| {
                            let name = event_target_value(&ev);
                            let mut ws = workspace.get_untracked();
                            if let Some(f) = ws.active_file_mut() {
                                f.name = name;
                            }
                            persist(ws);
                            bump_check();
                        }
                    />
                </div>
            </header>

            <main class="w-full px-3 pt-[8.5rem] pb-6">
                <HighlightedEditor
                    value=active_content
                    on_input=on_edit
                    on_keydown=move |ev: ev::KeyboardEvent| {
                        if ev.shift_key() && ev.key() == "Enter" {
                            ev.prevent_default();
                            run_script();
                        }
                    }
                    class="w-full min-h-[calc(100dvh-10rem)] font-mono text-sm bg-slate-950 border border-slate-800 rounded-lg p-4 resize-y text-slate-100"
                />

                <div node_ref=diagnostics_ref class="scroll-mt-36">
                {move || {
                    has_diagnostics().then(|| view! {
                        <section class="mt-4 rounded-lg border border-amber-900/50 bg-amber-950/30 p-3 text-sm">
                            <h2 class="font-semibold text-amber-100/90 mb-2">"Diagnostics"</h2>
                            {move || {
                                let err = error_banner.get();
                                (!err.is_empty()).then(|| view! {
                                    <p class="text-red-200 mb-2 break-words">{err}</p>
                                })
                            }}
                            <ul class="space-y-1">
                                {move || {
                                    diagnostics.get().into_iter().map(|d| view! {
                                        <li class="text-amber-200/90 break-words">
                                            {format!(
                                                "{}:{} [{}] {}",
                                                d.line, d.column, d.severity, d.message
                                            )}
                                        </li>
                                    }).collect_view()
                                }}
                            </ul>
                        </section>
                    })
                }}
                </div>

                <div node_ref=output_ref class="scroll-mt-36">
                {move || {
                    has_output().then(|| view! {
                        <section class="mt-4 rounded-lg border border-slate-700 bg-slate-950 p-3 text-sm font-mono">
                            <div class="flex items-center gap-2 mb-2 not-font-mono font-sans">
                                <h2 class="font-semibold text-slate-200">"Output"</h2>
                                <button
                                    type="button"
                                    class=move || {
                                        if output_tab.get() == "text" {
                                            "text-xs px-2 py-1 rounded bg-slate-700"
                                        } else {
                                            "text-xs px-2 py-1 rounded hover:bg-slate-800"
                                        }
                                    }
                                    on:click=move |_| set_output_tab.set("text".to_owned())
                                >
                                    "text"
                                </button>
                                <button
                                    type="button"
                                    class=move || {
                                        if output_tab.get() == "json" {
                                            "text-xs px-2 py-1 rounded bg-slate-700"
                                        } else {
                                            "text-xs px-2 py-1 rounded hover:bg-slate-800"
                                        }
                                    }
                                    on:click=move |_| set_output_tab.set("json".to_owned())
                                >
                                    "json"
                                </button>
                                <button
                                    type="button"
                                    class=move || {
                                        if output_tab.get() == "graph" {
                                            "text-xs px-2 py-1 rounded bg-slate-700"
                                        } else {
                                            "text-xs px-2 py-1 rounded hover:bg-slate-800"
                                        }
                                    }
                                    on:click=move |_| set_output_tab.set("graph".to_owned())
                                >
                                    "graph"
                                </button>
                            </div>
                            {move || {
                                let tab = output_tab.get();
                                if tab == "graph" {
                                    view! {
                                        <OutputGraphView outputs=result_outputs.get() />
                                    }
                                    .into_any()
                                } else {
                                    view! {
                                        <pre class="whitespace-pre-wrap break-words m-0">
                                            {if tab == "json" {
                                                result_json.get()
                                            } else {
                                                result_text.get()
                                            }}
                                        </pre>
                                    }
                                    .into_any()
                                }
                            }}
                        </section>
                    })
                }}
                </div>
            </main>
        </div>
    }
}
