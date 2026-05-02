use oboe_samples::debug_utils::{DebugLog, LogLevel, Trace};

fn main() {
    let mut log = DebugLog::default();
    log.push("AUDIO-APP", LogLevel::Info, "sample log");

    let mut trace = Trace::enabled();
    trace.begin_section("render");
    trace.end_section();

    println!("{:?}", log.records());
}
