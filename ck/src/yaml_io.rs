//! Path-aware YAML helpers. The string codec lives in `ck_core::yaml`; this adds
//! file I/O plus the `# yaml-language-server: $schema=...` header on write.

use std::path::Path;

use anyhow::{Context, Result};
use ck_core::yaml::{
    live_set_from_yaml_str, live_set_to_yaml_string, system_from_yaml_str, system_to_yaml_string,
    LIVE_SET_YAML_HEADER, SYSTEM_YAML_HEADER,
};
use ck_core::{LiveSet, System};

fn write_yaml(path: &Path, header: &str, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
    }
    let mut out = String::new();
    out.push_str(header);
    out.push('\n');
    out.push_str(body);
    std::fs::write(path, out).with_context(|| format!("writing {}", path.display()))
}

pub fn write_system(path: &Path, system: &System) -> Result<()> {
    let body = system_to_yaml_string(system).context("serialize System")?;
    write_yaml(path, SYSTEM_YAML_HEADER, &body)
}

pub fn write_live_set(path: &Path, live_set: &LiveSet) -> Result<()> {
    let body = live_set_to_yaml_string(live_set).context("serialize LiveSet")?;
    write_yaml(path, LIVE_SET_YAML_HEADER, &body)
}

pub fn read_system(path: &Path) -> Result<System> {
    let s = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    system_from_yaml_str(&s).with_context(|| format!("parsing System from {}", path.display()))
}

pub fn read_live_set(path: &Path) -> Result<LiveSet> {
    let s = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    live_set_from_yaml_str(&s).with_context(|| format!("parsing LiveSet from {}", path.display()))
}
