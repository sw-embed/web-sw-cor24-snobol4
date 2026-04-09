use gloo::timers::callback::Timeout;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, KeyboardEvent};
use yew::prelude::*;
use yew::NodeRef;

pub mod demos;
pub mod runner;

use demos::DEMOS;
use runner::Session;

const DEFAULT_DEMO: &str = "hello";
const DEFAULT_MAX_INSTRS: u64 = 200_000_000;
const BATCH_SIZE: u64 = 200_000;
const TICK_DELAY_MS: u32 = 0;

fn default_demo_index() -> usize {
    DEMOS
        .iter()
        .position(|d| d.name == DEFAULT_DEMO)
        .unwrap_or(0)
}

fn now_ms() -> f64 {
    js_sys::Date::now()
}

pub enum Msg {
    SelectDemo(usize),
    SourceChanged(String),
    DataChanged(String),
    StdinSubmit,
    Run,
    Tick,
    Stop,
    Reset,
    Clear,
    IncreaseBudget,
    KeyDown(KeyboardEvent),
}

pub struct App {
    selected: usize,
    source: String,
    data: String,
    output: String,
    status: String,
    error: bool,
    session: Option<Session>,
    running: bool,
    max_instrs: u64,
    started_at: f64,
    elapsed_ms: f64,
    /// True when last run halted on budget exhaustion (offer Increase Budget).
    budget_exhausted: bool,
    /// Ref to the stdin <input>. The field is uncontrolled (no `value`
    /// prop) so the perpetual Tick re-renders don't stomp what the
    /// user is typing -- on submit we read the live DOM value via this
    /// ref and clear it imperatively.
    stdin_ref: NodeRef,
}

impl App {
    fn load_demo(&mut self, idx: usize) {
        if let Some(demo) = DEMOS.get(idx) {
            self.selected = idx;
            self.source = demo.source.to_string();
            self.data = demo.data.unwrap_or("").to_string();
            self.output.clear();
            self.status = "idle".into();
            self.error = false;
            self.session = None;
            self.running = false;
            self.budget_exhausted = false;
            self.elapsed_ms = 0.0;
        }
    }

    fn start_run(&mut self, ctx: &Context<Self>) {
        self.session = Some(Session::new(&self.source, &self.data));
        self.running = true;
        self.error = false;
        self.budget_exhausted = false;
        self.output.clear();
        self.started_at = now_ms();
        self.elapsed_ms = 0.0;
        self.status = "running…".into();
        self.schedule_tick(ctx);
    }

    fn schedule_tick(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        Timeout::new(TICK_DELAY_MS, move || link.send_message(Msg::Tick)).forget();
    }

    fn finish(&mut self, status: String, error: bool) {
        self.running = false;
        self.status = status;
        self.error = error;
        self.elapsed_ms = now_ms() - self.started_at;
        if let Some(s) = &self.session {
            self.output = s.output().to_string();
        }
    }
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
            session: None,
            running: false,
            max_instrs: DEFAULT_MAX_INSTRS,
            started_at: 0.0,
            elapsed_ms: 0.0,
            budget_exhausted: false,
            stdin_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SelectDemo(i) => {
                self.load_demo(i);
                self.max_instrs = DEFAULT_MAX_INSTRS;
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
            Msg::StdinSubmit => {
                if let Some(session) = self.session.as_mut() {
                    if session.is_interactive() {
                        if let Some(input) = self.stdin_ref.cast::<HtmlInputElement>() {
                            for b in input.value().bytes() {
                                session.send_input_byte(b);
                            }
                            session.send_input_byte(b'\n');
                            input.set_value("");
                        }
                    }
                }
                false
            }
            Msg::Run => {
                self.max_instrs = DEFAULT_MAX_INSTRS;
                self.start_run(ctx);
                true
            }
            Msg::IncreaseBudget => {
                self.max_instrs = self.max_instrs.saturating_mul(4);
                self.start_run(ctx);
                true
            }
            Msg::Stop => {
                if self.running {
                    self.finish("stopped".into(), false);
                }
                true
            }
            Msg::Reset => {
                // Reload current demo from source, discarding edits.
                let idx = self.selected;
                self.running = false;
                self.session = None;
                self.load_demo(idx);
                self.max_instrs = DEFAULT_MAX_INSTRS;
                true
            }
            Msg::Clear => {
                self.output.clear();
                if !self.running {
                    self.status = "idle".into();
                    self.error = false;
                    self.budget_exhausted = false;
                }
                true
            }
            Msg::Tick => {
                if !self.running {
                    return false;
                }
                let Some(session) = self.session.as_mut() else {
                    self.running = false;
                    return true;
                };
                // Interactive sessions ignore the instruction budget --
                // the program is expected to spin on UART RX poll while
                // waiting for the user to type, so a fixed budget is
                // meaningless. Use Stop to end the session.
                let interactive = session.is_interactive();
                let batch = if interactive {
                    BATCH_SIZE
                } else {
                    let remaining = self.max_instrs.saturating_sub(session.instructions);
                    if remaining == 0 {
                        self.budget_exhausted = true;
                        let instrs = session.instructions;
                        self.finish(format!("halted (budget) — {} instrs", instrs), true);
                        return true;
                    }
                    remaining.min(BATCH_SIZE)
                };
                let result = session.tick(batch);
                if result.done {
                    let halted = session.halted;
                    let instrs = session.instructions;
                    let reason = session.stop_reason.clone();
                    self.finish(
                        if halted {
                            format!(
                                "done ({} instrs, {:.0} ms)",
                                instrs,
                                now_ms() - self.started_at
                            )
                        } else {
                            format!("{} ({} instrs)", reason, instrs)
                        },
                        !halted,
                    );
                } else {
                    // Live status update
                    self.output = session.output().to_string();
                    self.elapsed_ms = now_ms() - self.started_at;
                    self.status = format!(
                        "running… {} instrs, {:.0} ms",
                        session.instructions, self.elapsed_ms
                    );
                    self.schedule_tick(ctx);
                }
                true
            }
            Msg::KeyDown(e) => {
                if e.key() == "Enter" && (e.ctrl_key() || e.meta_key()) {
                    e.prevent_default();
                    ctx.link().send_message(Msg::Run);
                } else if e.key() == "Escape" && self.running {
                    e.prevent_default();
                    ctx.link().send_message(Msg::Stop);
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
        let on_stop = ctx.link().callback(|_| Msg::Stop);
        let on_reset = ctx.link().callback(|_| Msg::Reset);
        let on_clear = ctx.link().callback(|_| Msg::Clear);
        let on_inc = ctx.link().callback(|_| Msg::IncreaseBudget);
        let on_keydown = ctx.link().callback(Msg::KeyDown);
        let on_stdin_key = ctx.link().callback(|e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                Msg::StdinSubmit
            } else {
                Msg::KeyDown(e)
            }
        });

        let interactive_running = self
            .session
            .as_ref()
            .map(|s| s.is_interactive())
            .unwrap_or(false)
            && self.running;

        let status_class = if self.error {
            "status status-error"
        } else {
            "status"
        };
        let run_button = if self.running {
            html! { <button onclick={on_stop}>{ "Stop" }</button> }
        } else {
            html! { <button onclick={on_run}>{ "Run" }</button> }
        };

        html! {
            <>
            <a href="https://github.com/sw-embed/web-sw-cor24-snobol4" class="github-corner"
               aria-label="View source on GitHub" target="_blank">
                <svg width="80" height="80" viewBox="0 0 250 250" aria-hidden="true">
                    <path d="M0,0 L115,115 L130,115 L142,142 L250,250 L250,0 Z" />
                    <path d="M128.3,109.0 C113.8,99.7 119.0,89.6 119.0,89.6 C122.0,82.7 120.5,78.6 \
                        120.5,78.6 C119.2,72.0 123.4,76.3 123.4,76.3 C127.3,80.9 125.5,87.3 125.5,87.3 \
                        C122.9,97.6 130.6,101.9 134.4,103.2" fill="currentColor"
                        style="transform-origin:130px 106px;" class="octo-arm" />
                    <path d="M115.0,115.0 C114.9,115.1 118.7,116.5 119.8,115.4 L133.7,101.6 C136.9,99.2 \
                        139.9,98.4 142.2,98.6 C133.8,88.0 127.5,74.4 143.8,58.0 C148.5,53.4 154.0,51.2 \
                        159.7,51.0 C160.3,49.4 163.2,43.6 171.4,40.1 C171.4,40.1 176.1,42.5 178.8,56.2 \
                        C183.1,58.6 187.2,61.8 190.9,65.4 C194.5,69.0 197.7,73.2 200.1,77.6 C213.8,80.2 \
                        216.3,84.9 216.3,84.9 C212.7,93.1 206.9,96.0 205.4,96.6 C205.1,102.4 203.0,107.8 \
                        198.3,112.5 C181.9,128.9 168.3,122.5 157.7,114.1 C157.9,116.9 156.7,120.9 \
                        152.7,124.9 L141.0,136.5 C139.8,137.7 141.6,141.9 141.8,141.8 Z"
                        fill="currentColor" />
                </svg>
            </a>
            <main class="page" onkeydown={on_keydown.clone()}>
                <header class="chrome">
                    <h1>{ "web-sw-cor24-snobol4" }</h1>
                    <div class="controls">
                        <select onchange={on_demo} disabled={self.running}>
                            { for DEMOS.iter().enumerate().map(|(i, d)| html! {
                                <option value={i.to_string()} selected={i == self.selected}>
                                    { d.name }
                                </option>
                            })}
                        </select>
                        { run_button }
                        <button class="secondary" onclick={on_reset} disabled={self.running}>{ "Reset" }</button>
                        <button class="secondary" onclick={on_clear}>{ "Clear" }</button>
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
                        onkeydown={on_keydown.clone()}
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
                    <div class={status_class}>
                        { format!("status: {}", self.status) }
                        { if self.budget_exhausted {
                            html! {
                                <>
                                    { " — " }
                                    <a href="#" onclick={Callback::from(move |e: MouseEvent| {
                                        e.prevent_default();
                                    })}>
                                        <button class="link-btn" onclick={on_inc}>
                                            { "Increase budget 4×" }
                                        </button>
                                    </a>
                                </>
                            }
                        } else { html! {} }}
                    </div>
                    <pre class="out">{ &self.output }</pre>
                    { if interactive_running {
                        html! {
                            <div class="stdin-row">
                                <input
                                    ref={self.stdin_ref.clone()}
                                    type="text"
                                    class="stdin"
                                    placeholder="type a line and press Enter…"
                                    onkeydown={on_stdin_key}
                                />
                            </div>
                        }
                    } else { html! {} }}
                </section>
            </main>
            <footer>
                <span>{"MIT License"}</span>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <span>{"\u{00a9} 2026 Michael A Wright"}</span>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <a href="https://makerlisp.com" target="_blank">{"COR24-TB"}</a>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <a href="https://software-wrighter-lab.github.io/" target="_blank">{"Blog"}</a>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <a href="https://discord.com/invite/Ctzk5uHggZ" target="_blank">{"Discord"}</a>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <a href="https://www.youtube.com/@SoftwareWrighter" target="_blank">{"YouTube"}</a>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <a href="https://github.com/sw-embed/web-sw-cor24-snobol4/blob/main/docs/demos.md" target="_blank">{"Demo Documentation"}</a>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <a href="https://github.com/sw-embed/web-sw-cor24-snobol4/blob/main/CHANGES.md" target="_blank">{"Changes"}</a>
                <span class="footer-sep">{"\u{00b7}"}</span>
                <span>{ format!("{} \u{00b7} {} \u{00b7} {}",
                    env!("BUILD_HOST"),
                    env!("BUILD_SHA"),
                    env!("BUILD_TIMESTAMP"),
                ) }</span>
            </footer>
            </>
        }
    }
}
