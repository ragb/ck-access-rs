//! Command-line surface for the CK CLI.
//!
//! The CK exposes its settings as Yamaha address-based bulk blocks. This CLI
//! bundles them into two editable documents — the global **System** and a
//! **Live Set** patch — and dumps/syncs each as a single YAML file.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use ck_core::address::{BULK_FOOTER_BASE, BULK_HEADER_BASE, MASTER_EQ_BASE, SYSTEM_COMMON_BASE};
use ck_core::{identify_reply, voices, Category, LiveSet, MasterEq, System, SystemCommon};
use clap::{Parser, Subcommand};

use crate::midi::MidiSession;
use crate::yaml_io;

const REQUEST_TIMEOUT: Duration = Duration::from_millis(2500);

/// Top-level CLI.
#[derive(Parser, Debug)]
#[command(
    name = "ck",
    version,
    about = "Editor toolkit for the Yamaha CK61 / CK88 stage keyboards.",
    long_about = "Dump, edit, and re-sync the Yamaha CK61/CK88 over MIDI SysEx.\n\n\
        Settings are read/written as two human-editable YAML documents:\n  \
        • System    — global settings + Master EQ\n  \
        • Live Set  — one patch: common settings, EQ, 4 Zones, 3 Parts\n\n\
        Typical flow:\n  \
        ck ports                         # find your MIDI interface\n  \
        ck --port CK identity            # confirm the device + model\n  \
        ck --port CK dump --live-set -o patch.yaml\n  \
        # edit patch.yaml in your editor …\n  \
        ck --port CK sync --live-set -i patch.yaml --verify\n\n\
        Offline helpers (no device needed): show, lint, diff, voices, schema."
)]
pub struct Cli {
    /// MIDI port name substring for both directions. Override one side with
    /// --input-port / --output-port.
    #[arg(long, global = true)]
    pub port: Option<String>,

    /// MIDI input port name (substring). Overrides --port.
    #[arg(long, global = true)]
    pub input_port: Option<String>,

    /// MIDI output port name (substring). Overrides --port.
    #[arg(long, global = true)]
    pub output_port: Option<String>,

    /// Device number (0..=15) — must match the CK's MIDI Device Number setting.
    #[arg(long, global = true, default_value_t = 0)]
    pub device: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List available MIDI input and output ports.
    Ports,

    /// Probe the device with a Universal Identity Request (prints model + version).
    Identity,

    /// Read settings off the device into a YAML file.
    Dump {
        #[command(flatten)]
        target: Target,
        /// Output YAML file.
        #[arg(short = 'o', long)]
        output: PathBuf,
    },

    /// Send a YAML file's settings to the device.
    Sync {
        #[command(flatten)]
        target: Target,
        /// Input YAML file.
        #[arg(short = 'i', long)]
        input: PathBuf,
        /// After writing, read each block back and confirm the bytes match.
        #[arg(long)]
        verify: bool,
    },

    /// Pretty-print a YAML file (no device needed).
    Show { path: PathBuf },

    /// Validate a YAML file by decoding it through the typed model.
    Lint { path: PathBuf },

    /// Compare two YAML files of the same kind, field by field.
    Diff { left: PathBuf, right: PathBuf },

    /// List the built-in voices, optionally filtered by category.
    Voices {
        /// Category name substring (e.g. "piano", "organ"). Omit to list all.
        category: Option<String>,
    },

    /// Print the JSON Schema for one of the documents.
    Schema {
        /// Which schema: `system` or `live-set`.
        kind: String,
    },
}

/// Which document a dump/sync targets.
#[derive(clap::Args, Debug)]
#[group(required = true, multiple = false)]
pub struct Target {
    /// Target the global System settings.
    #[arg(long)]
    pub system: bool,
    /// Target the current Live Set patch.
    #[arg(long)]
    pub live_set: bool,
}

pub fn run(cli: Cli) -> Result<()> {
    let Cli {
        port,
        input_port,
        output_port,
        device,
        command,
    } = cli;
    let session_args = SessionArgs {
        port,
        input_port,
        output_port,
        device,
    };

    match command {
        Command::Ports => crate::midi::list_ports(),
        Command::Identity => identity(&mut open_session(&session_args)?),
        Command::Dump { target, output } => {
            let mut s = open_session(&session_args)?;
            if target.system {
                dump_system(&mut s, &output)
            } else {
                dump_live_set(&mut s, &output)
            }
        }
        Command::Sync {
            target,
            input,
            verify,
        } => {
            let mut s = open_session(&session_args)?;
            if target.system {
                sync_system(&mut s, &input, verify)
            } else {
                sync_live_set(&mut s, &input, verify)
            }
        }
        Command::Show { path } => show(&path),
        Command::Lint { path } => lint(&path),
        Command::Diff { left, right } => diff_files(&left, &right),
        Command::Voices { category } => list_voices(category.as_deref()),
        Command::Schema { kind } => emit_schema(&kind),
    }
}

// === session ===

struct SessionArgs {
    port: Option<String>,
    input_port: Option<String>,
    output_port: Option<String>,
    device: u8,
}

fn open_session(args: &SessionArgs) -> Result<MidiSession> {
    let port = args.port.as_deref();
    let input = args
        .input_port
        .as_deref()
        .or(port)
        .ok_or_else(|| anyhow!("--input-port or --port required (use `ck ports` to list)"))?;
    let output = args
        .output_port
        .as_deref()
        .or(port)
        .ok_or_else(|| anyhow!("--output-port or --port required (use `ck ports` to list)"))?;
    if args.device > 15 {
        bail!("--device {} out of range (0..=15)", args.device);
    }
    log::info!(
        "opening MIDI: input ~{input:?}, output ~{output:?}, device {}",
        args.device
    );
    Ok(MidiSession::open_with(input, output, args.device)?)
}

fn identity(session: &mut MidiSession) -> Result<()> {
    let reply = session.identity(REQUEST_TIMEOUT)?;
    let hex: String = reply
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ");
    println!("reply: {hex}");
    match identify_reply(&reply) {
        Some(model) => println!("  -> {} detected", model.name()),
        None => println!("  -> not recognised as a CK (family bytes didn't match)"),
    }
    Ok(())
}

// === dump ===

fn dump_system(session: &mut MidiSession, output: &Path) -> Result<()> {
    let common =
        SystemCommon::from_bytes(&session.request_bulk(SYSTEM_COMMON_BASE, REQUEST_TIMEOUT)?)?;
    let master_eq = MasterEq::from_bytes(&session.request_bulk(MASTER_EQ_BASE, REQUEST_TIMEOUT)?)?;
    yaml_io::write_system(output, &System { common, master_eq })?;
    println!("wrote {}", output.display());
    Ok(())
}

fn dump_live_set(session: &mut MidiSession, output: &Path) -> Result<()> {
    let live_set = read_live_set_from_device(session)?;
    yaml_io::write_live_set(output, &live_set)?;
    println!("wrote {}", output.display());
    Ok(())
}

fn read_live_set_from_device(session: &mut MidiSession) -> Result<LiveSet> {
    // A Bulk Request to the Bulk Header makes the CK stream the whole patch back
    // as a sequence of Bulk Dumps (common, EQ, zones, parts, …).
    let blocks = session.request_dump_sequence(BULK_HEADER_BASE, REQUEST_TIMEOUT)?;
    if blocks.is_empty() {
        bail!("no Live Set dump received (check --device and MIDI In/Out=On on the device)");
    }
    Ok(LiveSet::from_blocks(&blocks)?)
}

// === sync ===

fn sync_system(session: &mut MidiSession, input: &Path, verify: bool) -> Result<()> {
    let system = yaml_io::read_system(input)?;
    let blocks = [
        (
            SYSTEM_COMMON_BASE,
            system.common.to_bytes()?,
            "System Common",
        ),
        (MASTER_EQ_BASE, system.master_eq.to_bytes()?, "Master EQ"),
    ];
    for (addr, bytes, label) in blocks {
        write_block(session, addr, bytes, label, verify)?;
    }
    println!("sent System from {}", input.display());
    Ok(())
}

fn sync_live_set(session: &mut MidiSession, input: &Path, verify: bool) -> Result<()> {
    let ls = yaml_io::read_live_set(input)?;
    let blocks = ls.to_blocks()?; // validates zone/part counts

    // A Live Set is written as a framed sequence: Bulk Header, every content
    // block, then Bulk Footer (the order the device itself dumps them in).
    session.send_bulk(BULK_HEADER_BASE, Vec::new())?;
    for (addr, bytes) in &blocks {
        session.send_bulk(*addr, bytes.clone())?;
    }
    session.send_bulk(BULK_FOOTER_BASE, Vec::new())?;
    println!(
        "sent Live Set ({} blocks) from {}",
        blocks.len(),
        input.display()
    );

    if verify {
        std::thread::sleep(Duration::from_millis(80));
        let read_back = session.request_dump_sequence(BULK_HEADER_BASE, REQUEST_TIMEOUT)?;
        let mut got = read_back;
        got.sort_by_key(|(a, _)| *a);
        if got == blocks {
            println!("  verified: read-back matches ({} blocks)", blocks.len());
        } else {
            bail!("  verify FAILED: read-back differs from what we sent");
        }
    }
    Ok(())
}

fn write_block(
    session: &mut MidiSession,
    address: [u8; 3],
    bytes: Vec<u8>,
    label: &str,
    verify: bool,
) -> Result<()> {
    session.send_bulk(address, bytes.clone())?;
    if verify {
        std::thread::sleep(Duration::from_millis(40));
        let read_back = session.request_bulk(address, REQUEST_TIMEOUT)?;
        if read_back == bytes {
            println!("  {label}: verified ({} bytes)", bytes.len());
        } else {
            bail!("  {label}: verify FAILED (read-back differs from what we sent)");
        }
    }
    Ok(())
}

// === offline: show / lint / diff / voices / schema ===

fn show(path: &Path) -> Result<()> {
    let yaml =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let kind = if serde_yaml::from_str::<System>(&yaml).is_ok() {
        "System"
    } else if serde_yaml::from_str::<LiveSet>(&yaml).is_ok() {
        "Live Set"
    } else {
        "unrecognized — printing raw"
    };
    println!("# {} ({kind})", path.display());
    print!("{yaml}");
    Ok(())
}

fn lint(path: &Path) -> Result<()> {
    let yaml =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    match (
        serde_yaml::from_str::<System>(&yaml),
        serde_yaml::from_str::<LiveSet>(&yaml),
    ) {
        (Ok(s), _) => {
            s.common.to_bytes()?;
            s.master_eq.to_bytes()?;
            println!("OK: {} is a valid System", path.display());
        }
        (_, Ok(l)) => {
            l.common.to_bytes()?;
            l.eq.to_bytes()?;
            for z in &l.zones {
                z.to_bytes()?;
            }
            for p in &l.parts {
                p.to_bytes()?;
            }
            println!("OK: {} is a valid Live Set", path.display());
        }
        (Err(s), Err(l)) => bail!(
            "{} parses as neither:\n  - System:  {s}\n  - LiveSet: {l}",
            path.display()
        ),
    }
    Ok(())
}

fn diff_files(left: &Path, right: &Path) -> Result<()> {
    let a = std::fs::read_to_string(left).with_context(|| format!("reading {}", left.display()))?;
    let b =
        std::fs::read_to_string(right).with_context(|| format!("reading {}", right.display()))?;
    // Normalise through the typed model so formatting noise doesn't show up.
    if let (Ok(la), Ok(lb)) = (
        serde_yaml::from_str::<System>(&a),
        serde_yaml::from_str::<System>(&b),
    ) {
        return print_diff(&serde_yaml::to_string(&la)?, &serde_yaml::to_string(&lb)?);
    }
    if let (Ok(la), Ok(lb)) = (
        serde_yaml::from_str::<LiveSet>(&a),
        serde_yaml::from_str::<LiveSet>(&b),
    ) {
        return print_diff(&serde_yaml::to_string(&la)?, &serde_yaml::to_string(&lb)?);
    }
    bail!("both files must be the same kind (System or Live Set)");
}

fn print_diff(left: &str, right: &str) -> Result<()> {
    let la: Vec<&str> = left.lines().collect();
    let lb: Vec<&str> = right.lines().collect();
    let mut any = false;
    for i in 0..la.len().max(lb.len()) {
        let a = la.get(i).copied().unwrap_or("");
        let b = lb.get(i).copied().unwrap_or("");
        if a == b {
            continue;
        }
        any = true;
        let ka = a.split_once(':').map(|(k, _)| k.trim());
        let kb = b.split_once(':').map(|(k, _)| k.trim());
        match (ka, kb) {
            (Some(ka), Some(kb)) if ka == kb => {
                let va = a.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
                let vb = b.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
                println!("  {ka}: {va}  →  {vb}");
            }
            _ => {
                println!("- {a}");
                println!("+ {b}");
            }
        }
    }
    if !any {
        println!("(no differences)");
    }
    Ok(())
}

fn list_voices(filter: Option<&str>) -> Result<()> {
    let needle = filter.map(|s| s.to_lowercase());
    for i in 0..10u8 {
        let cat = Category::from_index(i).unwrap();
        if let Some(n) = &needle {
            if !cat.name().to_lowercase().contains(n) {
                continue;
            }
        }
        let (lo, hi) = cat.voice_range();
        println!("{} ({}–{}):", cat.name(), lo, hi);
        for v in lo..=hi {
            if let Some(name) = voices::voice_name(v) {
                println!("  {v:>3}  {name}");
            }
        }
    }
    Ok(())
}

fn emit_schema(kind: &str) -> Result<()> {
    let schema = match kind.to_ascii_lowercase().as_str() {
        "system" => ck_core::schema::system_schema(),
        "live-set" | "liveset" | "live_set" => ck_core::schema::live_set_schema(),
        other => bail!("unknown schema kind {other:?} (valid: system, live-set)"),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).context("serialize schema")?
    );
    Ok(())
}
