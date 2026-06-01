//! MIDI plumbing — open input + output, send CK [`Message`]s, and wait for the
//! matching reply. Bulk reads are request/response: we send a Bulk Request and
//! wait for the Bulk Dump the device returns at the same address.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::Result;
use ck_core::document::AddressedBlock;
use ck_core::{classify_inbound, InboundMessage, Message};
use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use thiserror::Error;

/// midir's backend `ConnectError` is `!Sync`; we stringify it at the boundary so
/// our own error stays `Send + Sync + 'static`.
#[derive(Debug, Error)]
pub enum MidiError {
    #[error("failed to open MIDI port: {reason}")]
    Open { reason: String },
    #[error("no MIDI {kind} port matches {needle:?}")]
    PortNotFound { kind: &'static str, needle: String },
    #[error("MIDI send failed: {reason}")]
    Send { reason: String },
    #[error("timed out after {0:?} waiting for a reply from the device")]
    Timeout(Duration),
}

pub fn list_ports() -> Result<()> {
    let mi = MidiInput::new("ck-list-in").map_err(open)?;
    let mo = MidiOutput::new("ck-list-out").map_err(open)?;
    println!("Input ports:");
    for p in mi.ports() {
        println!(
            "  - {}",
            mi.port_name(&p).unwrap_or_else(|_| "<unknown>".into())
        );
    }
    println!("\nOutput ports:");
    for p in mo.ports() {
        println!(
            "  - {}",
            mo.port_name(&p).unwrap_or_else(|_| "<unknown>".into())
        );
    }
    Ok(())
}

fn open(e: impl std::fmt::Display) -> MidiError {
    MidiError::Open {
        reason: e.to_string(),
    }
}

/// Active connection to one CK: input + output ports plus a background callback
/// forwarding inbound bytes to a channel.
pub struct MidiSession {
    output: MidiOutputConnection,
    rx: mpsc::Receiver<Vec<u8>>,
    _input: MidiInputConnection<()>,
    pub device: u8,
}

impl MidiSession {
    pub fn open_with(
        input_substring: &str,
        output_substring: &str,
        device: u8,
    ) -> Result<Self, MidiError> {
        let mi = MidiInput::new("ck-cli-in").map_err(open)?;
        let mo = MidiOutput::new("ck-cli-out").map_err(open)?;

        let in_port = find_port(mi.ports(), |p| mi.port_name(p), input_substring, "input")?;
        let out_port = find_port(mo.ports(), |p| mo.port_name(p), output_substring, "output")?;

        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let _input = mi
            .connect(
                &in_port,
                "ck",
                move |_t, msg, _| {
                    let _ = tx.send(msg.to_vec());
                },
                (),
            )
            .map_err(open)?;
        let output = mo.connect(&out_port, "ck").map_err(open)?;

        Ok(Self {
            output,
            rx,
            _input,
            device,
        })
    }

    fn drain(&self) {
        while self.rx.try_recv().is_ok() {}
    }

    fn send(&mut self, bytes: &[u8]) -> Result<(), MidiError> {
        self.output.send(bytes).map_err(|e| MidiError::Send {
            reason: e.to_string(),
        })
    }

    /// Send raw bytes (channel messages, etc.). Part of the session API surface.
    #[allow(dead_code)]
    pub fn send_raw(&mut self, bytes: &[u8]) -> Result<(), MidiError> {
        self.send(bytes)
    }

    /// Write a whole block to the device via a Bulk Dump.
    pub fn send_bulk(&mut self, address: [u8; 3], data: Vec<u8>) -> Result<(), MidiError> {
        let msg = Message::BulkDump {
            device: self.device,
            address,
            data,
        };
        self.send(&msg.encode())
    }

    /// Write a single parameter via a Parameter Change. Part of the session API.
    #[allow(dead_code)]
    pub fn send_parameter(&mut self, address: [u8; 3], data: Vec<u8>) -> Result<(), MidiError> {
        let msg = Message::ParameterChange {
            device: self.device,
            address,
            data,
        };
        self.send(&msg.encode())
    }

    /// Send a Bulk Request and wait for the Bulk Dump the device returns at the
    /// same address. Returns the dumped block's data bytes.
    pub fn request_bulk(
        &mut self,
        address: [u8; 3],
        timeout: Duration,
    ) -> Result<Vec<u8>, MidiError> {
        self.drain();
        let req = Message::BulkRequest {
            device: self.device,
            address,
        };
        self.send(&req.encode())?;
        let deadline = Instant::now() + timeout;
        loop {
            let raw = self.recv_until(deadline, timeout)?;
            if let InboundMessage::BulkDump {
                address: a, data, ..
            } = classify_inbound(&raw)
            {
                if a == address {
                    return Ok(data);
                }
            }
            // Echoes, parameter changes, clock — keep listening.
        }
    }

    /// Send a Bulk Request to a Bulk Header address and collect the multi-block
    /// dump the device streams back, up to and including the Bulk Footer.
    /// Returns the content blocks as `(address, data)` (header/footer excluded).
    pub fn request_dump_sequence(
        &mut self,
        header: [u8; 3],
        timeout: Duration,
    ) -> Result<Vec<AddressedBlock>, MidiError> {
        self.drain();
        self.send(
            &Message::BulkRequest {
                device: self.device,
                address: header,
            }
            .encode(),
        )?;
        let mut blocks = Vec::new();
        // After each message, allow a short gap for the next; stop on the footer.
        let mut deadline = Instant::now() + timeout;
        loop {
            let raw = match self.recv_until(deadline, timeout) {
                Ok(b) => b,
                Err(MidiError::Timeout(_)) if !blocks.is_empty() => break,
                Err(e) => return Err(e),
            };
            if let InboundMessage::BulkDump { address, data, .. } = classify_inbound(&raw) {
                let is_footer = matches!(address, [0x0F, _, _]);
                let is_header = matches!(address, [0x0E, _, _]);
                if !is_header && !is_footer {
                    blocks.push((address, data));
                }
                if is_footer {
                    break;
                }
                deadline = Instant::now() + Duration::from_millis(600);
            }
        }
        Ok(blocks)
    }

    /// Send a Universal Identity Request and return the raw Identity Reply.
    pub fn identity(&mut self, timeout: Duration) -> Result<Vec<u8>, MidiError> {
        self.drain();
        self.send(&ck_core::identity_request())?;
        let deadline = Instant::now() + timeout;
        loop {
            let raw = self.recv_until(deadline, timeout)?;
            if matches!(classify_inbound(&raw), InboundMessage::IdentityReply { .. }) {
                return Ok(raw);
            }
        }
    }

    fn recv_until(&self, deadline: Instant, timeout: Duration) -> Result<Vec<u8>, MidiError> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(MidiError::Timeout(timeout));
        }
        match self.rx.recv_timeout(remaining) {
            Ok(b) => Ok(b),
            Err(mpsc::RecvTimeoutError::Timeout) => Err(MidiError::Timeout(timeout)),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(MidiError::Send {
                reason: "input channel disconnected".into(),
            }),
        }
    }
}

fn find_port<P>(
    ports: Vec<P>,
    name_of: impl Fn(&P) -> Result<String, midir::PortInfoError>,
    needle: &str,
    kind: &'static str,
) -> Result<P, MidiError> {
    ports
        .into_iter()
        .find(|p| {
            name_of(p)
                .map(|n| n.to_lowercase().contains(&needle.to_lowercase()))
                .unwrap_or(false)
        })
        .ok_or_else(|| MidiError::PortNotFound {
            kind,
            needle: needle.to_string(),
        })
}
