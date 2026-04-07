use yew::prelude::*;

pub mod demos;

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <main class="page">
                <header class="chrome">
                    <h1>{ "web-sw-cor24-snobol4" }</h1>
                    <div class="controls">
                        <select disabled=true>
                            <option>{ "hello" }</option>
                        </select>
                        <button disabled=true>{ "Run" }</button>
                    </div>
                </header>
                <section class="panel">
                    <label>{ "source (.sno)" }</label>
                    <textarea class="src" rows="12" disabled=true />
                </section>
                <section class="panel">
                    <label>{ "input data (optional)" }</label>
                    <textarea class="data" rows="4" disabled=true />
                </section>
                <section class="panel">
                    <div class="status">{ "status: idle" }</div>
                    <pre class="out"></pre>
                </section>
            </main>
        }
    }
}
