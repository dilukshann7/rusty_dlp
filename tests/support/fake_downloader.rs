//! Offline integration-test executable. Not included in normal builds or release packages.
use std::{
    io::{self, Write},
    path::PathBuf,
    time::Duration,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--descendant") {
        std::thread::sleep(Duration::from_millis(900));
        std::fs::write(&args[1], "descendant survived").unwrap();
        std::thread::sleep(Duration::from_secs(20));
        return;
    }
    if args.contains(&"--version".into()) {
        println!("2026.08.19");
        return;
    }
    if args.contains(&"--dump-single-json".into()) {
        println!(
            "{{\"id\":\"fake\",\"title\":\"Offline fixture\",\"formats\":[{{\"format_id\":\"18\",\"ext\":\"mp4\",\"height\":360,\"vcodec\":\"avc1\",\"acodec\":\"mp4a\"}}]}}"
        );
        return;
    }
    let get = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|index| args.get(index + 1))
            .cloned()
    };
    let folder = PathBuf::from(get("--paths").unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&folder).unwrap();
    let scenario = args.last().cloned().unwrap_or_default();
    if scenario.contains("descendant") {
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--descendant",
                folder.join("descendant-marker").to_str().unwrap(),
            ])
            .spawn()
            .unwrap();
        std::thread::sleep(Duration::from_secs(20));
        return;
    }
    if scenario.contains("slow") {
        std::thread::sleep(Duration::from_secs(20));
    }
    if scenario.contains("flood") {
        for _ in 0..30000 {
            eprintln!(
                "A diagnostic line that must be drained without blocking or retaining unbounded history"
            );
        }
    }
    println!("RDL_DOWNLOAD");
    print!("RDL_PROG");
    io::stdout().flush().unwrap();
    std::thread::sleep(Duration::from_millis(10));
    println!("RESS:{{\"status\":\"finished\",\"downloaded_bytes\":12,\"total_bytes\":12}}");
    println!("RDL_POST:{{\"status\":\"started\"}}");
    if scenario.contains("fail") {
        eprintln!("Deliberate fixture failure after progress finished");
        std::process::exit(2);
    }
    if scenario.contains("missing") {
        return;
    }
    let output = folder.join("fixture.mp4");
    std::fs::write(&output, "fixture bytes").unwrap();
    println!(
        "RDL_OUTPUT:{}",
        serde_json::to_string(&output.to_string_lossy()).unwrap()
    );
}
