use oort_analyzer::AnalyzerAgent;
use yew_agent::PrivateAgent;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Warn).expect("initializing logging");
    log::info!("starting analyzer");
    AnalyzerAgent::register();
}