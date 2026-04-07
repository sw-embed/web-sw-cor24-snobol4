use web_sys::{HtmlSelectElement, HtmlTextAreaElement, KeyboardEvent};
use yew::prelude::*;

pub mod demos;
pub mod runner;

use demos::DEMOS;

const DEFAULT_DEMO: &str = "hello";
const MAX_INSTRS: u64 = 200_000_000;

fn default_demo_index() -> usize {
    DEMOS
        .iter()
        .position(|d| d.name == DEFAULT_DEMO)
        .unwrap_or(0)
}

pub enum Msg {
    SelectDemo(usize),
    SourceChanged(String),
    DataChanged(String),
    Run,
    SourceKeyDown(KeyboardEvent),
}

pub struct App {
    selected: usize,
    source: String,
    data: String,
    output: String,
    status: String,
    error: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let idx = default_demo_index();
        let demo = &DEMOS[idx];
        Self {
            selected: idx,
            source: demo.source.to_string(),
            data: demo.data.unwrap_or("").to_string(),
            output: String::new(),
            status: "idle".into(),
            error: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SelectDemo(i) => {
                if let Some(demo) = DEMOS.get(i) {
                    self.selected = i;
                    self.source = demo.source.to_string();
                    self.data = demo.data.unwrap_or("").to_string();
                    self.output.clear();
                    self.status = "idle".into();
                    self.error = false;
                }
                true
            }
            Msg::SourceChanged(v) => {
                self.source = v;
                false
            }
            Msg::DataChanged(v) => {
                self.data = v;
                false
            }
            Msg::Run => {
                self.status = "running…".into();
                self.error = false;
                match runner::run_snobol4(&self.source, &self.data, MAX_INSTRS) {
                    Ok(result) => {
                        self.output = result.output;
                        self.status = format!(
                            "{} ({} instrs)",
                            if result.halted { "done" } else { result.stop_reason.as_str() },
                            result.instructions,
                        );
                        self.error = !result.halted;
                    }
                    Err(e) => {
                        self.output = format!("error: {}", e);
                        self.status = "error".into();
                        self.error = true;
                    }
                }
                true
            }
            Msg::SourceKeyDown(e) => {
                if e.key() == "Enter" && (e.ctrl_key() || e.meta_key()) {
                    e.prevent_default();
                    _ctx.link().send_message(Msg::Run);
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_demo = ctx.link().callback(|e: Event| {
            let target: HtmlSelectElement = e.target_unchecked_into();
            let idx: usize = target.value().parse().unwrap_or(0);
            Msg::SelectDemo(idx)
        });
        let on_src = ctx.link().callback(|e: InputEvent| {
            let target: HtmlTextAreaElement = e.target_unchecked_into();
            Msg::SourceChanged(target.value())
        });
        let on_data = ctx.link().callback(|e: InputEvent| {
            let target: HtmlTextAreaElement = e.target_unchecked_into();
            Msg::DataChanged(target.value())
        });
        let on_run = ctx.link().callback(|_| Msg::Run);
        let on_keydown = ctx.link().callback(Msg::SourceKeyDown);

        let status_class = if self.error { "status status-error" } else { "status" };

        html! {
            <main class="page">
                <header class="chrome">
                    <h1>{ "web-sw-cor24-snobol4" }</h1>
                    <div class="controls">
                        <select onchange={on_demo}>
                            { for DEMOS.iter().enumerate().map(|(i, d)| html! {
                                <option value={i.to_string()} selected={i == self.selected}>
                                    { d.name }
                                </option>
                            })}
                        </select>
                        <button onclick={on_run}>{ "Run" }</button>
                    </div>
                </header>
                <section class="panel">
                    <label>{ "source (.sno)" }</label>
                    <textarea
                        class="src"
                        rows="14"
                        spellcheck="false"
                        value={self.source.clone()}
                        oninput={on_src}
                        onkeydown={on_keydown}
                    />
                </section>
                <section class="panel">
                    <label>{ "input data (optional)" }</label>
                    <textarea
                        class="data"
                        rows="4"
                        spellcheck="false"
                        value={self.data.clone()}
                        oninput={on_data}
                    />
                </section>
                <section class="panel">
                    <div class={status_class}>{ format!("status: {}", self.status) }</div>
                    <pre class="out">{ &self.output }</pre>
                </section>
            </main>
        }
    }
}
