//! # Youki
//! Container Runtime written in Rust, inspired by [railcar](https://github.com/oracle/railcar)
//! This crate provides a container runtime which can be used by a high-level container runtime to run containers.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::Clap;

use youki::commands::create;
use youki::commands::delete;
use youki::commands::events;
use youki::commands::exec;
use youki::commands::info;
use youki::commands::kill;
use youki::commands::list;
use youki::commands::pause;
use youki::commands::ps;
use youki::commands::resume;
use youki::commands::run;
use youki::commands::spec_json;
use youki::commands::start;
use youki::commands::state;
use youki::rootless::should_use_rootless;

// High-level commandline option definition
// This takes global options as well as individual commands as specified in [OCI runtime-spec](https://github.com/opencontainers/runtime-spec/blob/master/runtime.md)
// Also check [runc commandline documentation](https://github.com/opencontainers/runc/blob/master/man/runc.8.md) for more explanation
#[derive(Clap, Debug)]
#[clap(version = "0.0.0", author = "youki team")]
struct Opts {
    /// root directory to store container state
    #[clap(short, long, default_value = "/run/youki")]
    root: PathBuf,
    #[clap(short, long)]
    log: Option<PathBuf>,
    #[clap(long)]
    log_format: Option<String>,
    /// Enable systemd cgroup manager, rather then use the cgroupfs directly.
    #[clap(short, long)]
    systemd_cgroup: bool,
    /// command to actually manage container
    #[clap(subcommand)]
    subcmd: SubCommand,
}

// Subcommands accepted by Youki, confirming with [OCI runtime-spec](https://github.com/opencontainers/runtime-spec/blob/master/runtime.md)
// Also for a short information, check [runc commandline documentation](https://github.com/opencontainers/runc/blob/master/man/runc.8.md)
#[derive(Clap, Debug)]
enum SubCommand {
    #[clap(version = "0.0.0", author = "youki team")]
    Create(create::Create),
    #[clap(version = "0.0.0", author = "youki team")]
    Start(start::Start),
    #[clap(version = "0.0.0", author = "youki team")]
    Run(run::Run),
    #[clap(version = "0.0.0", author = "youki team")]
    Exec(exec::Exec),
    #[clap(version = "0.0.0", author = "youki team")]
    Kill(kill::Kill),
    #[clap(version = "0.0.0", author = "youki team")]
    Delete(delete::Delete),
    #[clap(version = "0.0.0", author = "youki team")]
    State(state::State),
    #[clap(version = "0.0.0", author = "youki team")]
    Info(info::Info),
    #[clap(version = "0.0.0", author = "youki team")]
    Spec(spec_json::SpecJson),
    #[clap(version = "0.0.0", author = "youki team")]
    List(list::List),
    #[clap(version = "0.0.0", author = "youki team")]
    Pause(pause::Pause),
    #[clap(version = "0.0.0", author = "youki team")]
    Resume(resume::Resume),
    #[clap(version = "0.0.0", author = "youki team")]
    Events(events::Events),
    #[clap(version = "0.0.0", author = "youki team", setting=clap::AppSettings::AllowLeadingHyphen)]
    Ps(ps::Ps),
}

/// This is the entry point in the container runtime. The binary is run by a high-level container runtime,
/// with various flags passed. This parses the flags, creates and manages appropriate resources.
fn main() -> Result<()> {
    let opts = Opts::parse();

    if let Err(e) = youki::logger::init(opts.log) {
        eprintln!("log init failed: {:?}", e);
    }

    let root_path = if should_use_rootless() && opts.root.eq(&PathBuf::from("/run/youki")) {
        PathBuf::from("/tmp/rootless")
    } else {
        PathBuf::from(&opts.root)
    };
    fs::create_dir_all(&root_path)?;

    let systemd_cgroup = opts.systemd_cgroup;

    match opts.subcmd {
        SubCommand::Create(create) => create.exec(root_path, systemd_cgroup),
        SubCommand::Start(start) => start.exec(root_path),
        SubCommand::Run(run) => run.exec(root_path, systemd_cgroup),
        SubCommand::Exec(exec) => exec.exec(root_path),
        SubCommand::Kill(kill) => kill.exec(root_path),
        SubCommand::Delete(delete) => delete.exec(root_path, systemd_cgroup),
        SubCommand::State(state) => state.exec(root_path),
        SubCommand::Info(info) => info.exec(),
        SubCommand::List(list) => list.exec(root_path),
        SubCommand::Spec(spec) => spec.exec(),
        SubCommand::Pause(pause) => pause.exec(root_path, systemd_cgroup),
        SubCommand::Resume(resume) => resume.exec(root_path, systemd_cgroup),
        SubCommand::Events(events) => events.exec(root_path),
        SubCommand::Ps(ps) => ps.exec(root_path),
    }
}
