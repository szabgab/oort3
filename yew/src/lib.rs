pub mod api;
pub mod game;
pub mod sim_agent;
pub mod ui;

use chrono::NaiveDateTime;
use game::Game;
use oort_simulator::scenario;
use rand::Rng;
use rbtag::{BuildDateTime, BuildInfo};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use yew::agent::{Bridge, Bridged};
use yew::prelude::*;
use yew::services::render::{RenderService, RenderTask};

#[derive(BuildDateTime, BuildInfo)]
struct BuildTag;

pub fn version() -> String {
    let build_time = NaiveDateTime::from_timestamp(
        BuildTag {}
            .get_build_timestamp()
            .parse::<i64>()
            .unwrap_or(0),
        0,
    );

    let commit = BuildTag {}.get_build_commit();

    if commit.contains("dirty") {
        commit.to_string()
    } else {
        format!("{} {}", build_time.format("%Y%m%d.%H%M%S"), commit)
    }
}

pub enum Msg {
    Render,
    SelectScenario(String),
    KeyEvent(web_sys::KeyboardEvent),
    WheelEvent(web_sys::WheelEvent),
    ReceivedSimAgentResponse(sim_agent::Response),
    RequestSnapshot,
    EditorAction(String),
}

pub struct Model {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    render_task: RenderTask,
    game: Game,
    scenario_name: String,
    sim_agent: Box<dyn Bridge<sim_agent::SimAgent>>,
    editor_ref: NodeRef,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Msg::SelectScenario("welcome".to_string()));
        let link2 = link.clone();
        let render_task = RenderService::request_animation_frame(Callback::from(move |_| {
            link2.send_message(Msg::Render)
        }));
        let game = game::create(link.callback(|_| Msg::RequestSnapshot));
        let sim_agent = sim_agent::SimAgent::bridge(link.callback(Msg::ReceivedSimAgentResponse));
        Self {
            link,
            render_task,
            game,
            scenario_name: String::new(),
            sim_agent,
            editor_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Render => {
                self.game.render();
                let link2 = self.link.clone();
                self.render_task =
                    RenderService::request_animation_frame(Callback::from(move |_| {
                        link2.send_message(Msg::Render)
                    }));
                false
            }
            Msg::SelectScenario(scenario_name) => {
                self.scenario_name = scenario_name;
                let code = self.game.get_saved_code(&self.scenario_name);
                api::editor::set_text(&code);
                let seed = rand::thread_rng().gen();
                self.game.start(&self.scenario_name, "");
                self.sim_agent.send(sim_agent::Request::StartScenario {
                    scenario_name: self.scenario_name.to_owned(),
                    seed,
                    code: String::new(),
                });
                false
            }
            Msg::EditorAction(ref action) if action == "execute" => {
                let code = api::editor::get_text();
                self.game.save_code(&self.scenario_name, &code);
                let seed = rand::thread_rng().gen();
                self.game.start(&self.scenario_name, &code);
                self.sim_agent.send(sim_agent::Request::StartScenario {
                    scenario_name: self.scenario_name.to_owned(),
                    seed,
                    code: code.to_owned(),
                });
                false
            }
            Msg::EditorAction(ref action) if action == "load-initial-code" => {
                let code = self.game.get_initial_code(&self.scenario_name);
                api::editor::set_text(&code);
                false
            }
            Msg::EditorAction(ref action) if action == "load-solution-code" => {
                let code = self.game.get_solution_code(&self.scenario_name);
                api::editor::set_text(&code);
                false
            }
            Msg::EditorAction(action) => {
                log::info!("Got unexpected editor action {}", action);
                false
            }
            Msg::KeyEvent(e) => {
                self.game.on_key_event(e);
                false
            }
            Msg::WheelEvent(e) => {
                self.game.on_wheel_event(e);
                false
            }
            Msg::ReceivedSimAgentResponse(sim_agent::Response::Snapshot { snapshot }) => {
                self.game.on_snapshot(snapshot);
                false
            }
            Msg::RequestSnapshot => {
                self.sim_agent
                    .send(sim_agent::Request::Snapshot { nonce: 0 });
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        fn render_option(name: String) -> Html {
            html! { <option name={name.clone()}>{name}</option> }
        }

        let select_scenario_cb = self.link.callback(|data: ChangeData| match data {
            ChangeData::Select(elem) => Msg::SelectScenario(elem.value()),
            _ => unreachable!(),
        });

        let key_event_cb = self.link.callback(Msg::KeyEvent);
        let wheel_event_cb = self.link.callback(Msg::WheelEvent);

        let username = self.game.get_username(&self.game.get_userid());

        html! {
        <>
            <canvas id="glcanvas"
                tabindex="1"
                onkeydown=key_event_cb.clone()
                onkeyup=key_event_cb
                onwheel=wheel_event_cb />
            <div id="editor" ref=self.editor_ref.clone() />
            <div id="status"></div>
            <div id="toolbar">
                <div class="toolbar-elem title">{ "Oort" }</div>
                <div class="toolbar-elem right">
                    <select name="scenario" id="scenario" onchange=select_scenario_cb>
                        { for scenario::list().iter().cloned().map(render_option) }
                    </select>
                </div>
                <div class="toolbar-elem right"><a id="doc_link" href="#">{ "Documentation" }</a></div>
                <div class="toolbar-elem right"><a href="http://github.com/rlane/oort3" target="_none">{ "GitHub" }</a></div>
                <div class="toolbar-elem right"><a href="https://trello.com/b/PLQYouu8" target="_none">{ "Trello" }</a></div>
                <div class="toolbar-elem right"><a href="https://discord.gg/vYyu9EhkKH" target="_none">{ "Discord" }</a></div>
                <div id="username" class="toolbar-elem right" title="Your username">{ username }</div>
            </div>
            <div id="overlay">
                <div id="splash-overlay" class="inner-overlay"></div>
                <div id="doc-overlay" class="inner-overlay">
                    <h1>{ "Quick Reference" }</h1>
                    { "Press Escape to close. File bugs on " }<a href="http://github.com/rlane/oort3/issues" target="_none">{ "GitHub" }</a>{ "." }<br />

                    <h2>{ "Basics" }</h2>
                    { "Select a scenario from the list in the top-right of the page." }<br/>
                    { "Press Ctrl-Enter in the editor to run the scenario with a new version of your code." }<br/>
                    { "The game calls your <code>tick()</code> function 60 times per second." }
                </div>
                <div id="mission-complete-overlay" class="inner-overlay">
                </div>
            </div>
        </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            if let Some(editor_div) = self.editor_ref.cast::<web_sys::HtmlElement>() {
                let cb = self.link.callback(Msg::EditorAction);
                let closure =
                    Closure::wrap(Box::new(move |action| cb.emit(action)) as Box<dyn FnMut(_)>);
                api::editor::initialize(editor_div, &closure);
                closure.forget();
            }
        }
    }
}

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<Model>();
    Ok(())
}