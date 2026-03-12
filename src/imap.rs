// This file is part of Mirador, a CLI to watch mailbox changes.
//
// Copyright (C) 2024-2026 Clément DOUIN <pimalaya.org@posteo.net>
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU Affero General Public License
// as published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public
// License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

use std::{
    collections::{BTreeMap, HashSet},
    io::ErrorKind::*,
    num::NonZeroU32,
    process::exit,
    time::Duration,
};

use anyhow::Result;
use clap::Parser;
use io_imap::{
    coroutines::{fetch::*, idle::*, select::*},
    types::{
        core::Vec1,
        envelope::Address,
        fetch::{MessageDataItem, MessageDataItemName},
        flag::FlagFetch,
        mailbox::Mailbox,
        response::Data,
    },
};
use io_stream::{coroutines::read::ReadStream, io::StreamIo, runtimes::std::handle};
use log::{debug, info};
use pimalaya_toolbox::{
    sasl::Sasl,
    stream::{imap::ImapSession, Tls},
};
use rfc2047_decoder::{Decoder, RecoverStrategy};
use url::Url;

/// IMAP CLI (requires the `imap` cargo feature).
///
/// This command gives you access to the IMAP CLI API, and allows you
/// to manage IMAP mailboxes, envelopes, flags, messages etc.
#[derive(Debug, Parser)]
pub struct ImapWatchCommand {
    /// The mailbox to watch changes for.
    #[arg(default_value = "INBOX")]
    pub mailbox: String,
}

impl ImapWatchCommand {
    pub fn execute(self, url: Url, tls: Tls, starttls: bool, sasl: Sasl) -> Result<()> {
        let mut imap = ImapSession::new(url.clone(), tls.clone(), starttls, sasl.clone())?;
        imap.stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        let mbox = Mailbox::try_from(self.mailbox)?;
        let mut cache = Cache::new(url, tls, starttls, sasl, mbox.clone())?;

        let mut arg = None;
        let mut coroutine = ImapSelect::new(imap.context, mbox);

        imap.context = loop {
            match coroutine.resume(arg.take()) {
                ImapSelectResult::Io { io } => arg = Some(handle(&mut imap.stream, io)?),
                ImapSelectResult::Ok { context, .. } => break context,
                ImapSelectResult::Err { err, .. } => Err(err)?,
            }
        };

        let mut arg = None;
        let (mut coroutine, idle) = ImapIdle::new(imap.context);

        ctrlc::set_handler({
            let idle = idle.clone();
            move || {
                if idle.is_done() {
                    println!("Termination received twice, exiting…");
                    exit(0);
                } else {
                    println!("Termination received, waiting for read to time out…");
                    idle.done()
                }
            }
        })?;

        println!("Running IMAP IDLE command…");

        while !idle.is_done() {
            loop {
                match coroutine.resume(arg.take()) {
                    ImapIdleResult::Io { io } => match handle(&mut imap.stream, io) {
                        Ok(out) => {
                            arg = Some(out);
                        }
                        Err(err) if err.kind() == WouldBlock || err.kind() == TimedOut => {
                            let io = Err(vec![0; ReadStream::DEFAULT_CAPACITY]);
                            arg = Some(StreamIo::Read(io))
                        }
                        Err(err) => Err(err)?,
                    },
                    ImapIdleResult::Data { data, .. } => {
                        for data in data {
                            match data {
                                Data::Exists(n) => cache.apply_exists(n)?,
                                Data::Expunge(n) => cache.apply_expunge(n),
                                Data::Fetch { seq, items } => cache.apply_fetch(seq, items),
                                _ => continue,
                            }
                        }
                    }
                    ImapIdleResult::Ok { .. } => break,
                    ImapIdleResult::Err { err, .. } => Err(err)?,
                };
            }
        }

        println!("IMAP IDLE successfully terminated");
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct Envelope {
    pub seq: u32,
    pub uid: u32,
    pub flags: HashSet<String>,
    pub from: String,
    pub subject: String,
}

impl Envelope {
    fn new(seq: NonZeroU32, items: Vec1<MessageDataItem>) -> Self {
        let mut envelope = Envelope::default();
        envelope.seq = seq.get();

        for item in items.into_iter() {
            match item {
                MessageDataItem::Uid(uid) => {
                    envelope.uid = uid.get();
                }
                MessageDataItem::Flags(flags) => {
                    envelope.flags = flags
                        .into_iter()
                        .map(|f| match f {
                            FlagFetch::Flag(f) => format!("{f}"),
                            FlagFetch::Recent => "\\Recent".into(),
                        })
                        .collect();
                }
                MessageDataItem::Envelope(env) => {
                    if let Some(s) = &env.subject.0 {
                        envelope.subject =
                            decode_mime(String::from_utf8_lossy(s.as_ref()).as_ref());
                    }

                    envelope.from = format_addresses_short(&env.from);
                }
                _ => continue,
            }
        }

        envelope
    }
}

#[derive(Debug)]
struct Cache {
    imap: Option<ImapSession>,
    by_uid: BTreeMap<u32, Envelope>,
    seq_to_uid: Vec<u32>,
}

impl Cache {
    fn new(url: Url, tls: Tls, starttls: bool, sasl: Sasl, mbox: Mailbox<'static>) -> Result<Self> {
        info!("build envelopes cache from scratch (1:*)");
        let mut imap = ImapSession::new(url, tls, starttls, sasl)?;

        // SELECT

        let mut arg = None;
        let mut coroutine = ImapSelect::new(imap.context, mbox.clone());

        imap.context = loop {
            match coroutine.resume(arg.take()) {
                ImapSelectResult::Io { io } => arg = Some(handle(&mut imap.stream, io)?),
                ImapSelectResult::Ok { context, .. } => break context,
                ImapSelectResult::Err { err, .. } => Err(err)?,
            }
        };

        // FETCH 1:*

        let seq = "1:*".try_into().unwrap();
        let items = vec![
            MessageDataItemName::Uid,
            MessageDataItemName::Envelope,
            MessageDataItemName::Flags,
        ];

        let mut arg = None;
        let mut coroutine = ImapFetch::new(imap.context, seq, items.into(), false);

        let data = loop {
            match coroutine.resume(arg.take()) {
                ImapFetchResult::Io { io } => arg = Some(handle(&mut imap.stream, io)?),
                ImapFetchResult::Ok { context, data } => {
                    imap.context = context;
                    break data;
                }
                ImapFetchResult::Err { err, .. } => Err(err)?,
            }
        };

        let mut by_uid = BTreeMap::new();
        let mut seq_to_uid = vec![0; data.len()];

        for (seq, items) in data {
            let envelope = Envelope::new(seq, items);
            let uid = envelope.uid;
            let seq = envelope.seq as usize - 1;
            by_uid.insert(uid, envelope);
            seq_to_uid[seq] = uid;
        }

        Ok(Self {
            imap: Some(imap),
            by_uid,
            seq_to_uid,
        })
    }

    fn apply_exists(&mut self, n: u32) -> Result<()> {
        let mut imap = self.imap.take().unwrap();

        let prev_n = self.seq_to_uid.len() + 1;
        let seq = format!("{prev_n}:{n}").as_str().try_into().unwrap();

        let items = vec![
            MessageDataItemName::Uid,
            MessageDataItemName::Envelope,
            MessageDataItemName::Flags,
        ];

        let mut arg = None;
        let mut coroutine = ImapFetch::new(imap.context, seq, items.into(), false);

        let mut data = loop {
            match coroutine.resume(arg.take()) {
                ImapFetchResult::Io { io } => arg = Some(handle(&mut imap.stream, io)?),
                ImapFetchResult::Ok { context, data } => {
                    imap.context = context;
                    break data;
                }
                ImapFetchResult::Err { err, .. } => Err(err)?,
            }
        };

        // keep only FETCH with UID, ENVELOPE and FLAGS items with SEQ
        // in range
        data.retain(|seq, items| {
            if (seq.get() as usize) < prev_n {
                return false;
            }

            if seq.get() > n {
                return false;
            }

            let mut has_uid = false;
            let mut has_envelope = false;
            let mut has_flags = false;

            for item in items.as_ref() {
                match item {
                    MessageDataItem::Uid(_) => has_uid = true,
                    MessageDataItem::Envelope(_) => has_envelope = true,
                    MessageDataItem::Flags(_) => has_flags = true,
                    _ => continue,
                }
            }

            has_uid && has_envelope && has_flags
        });

        self.seq_to_uid
            .resize(self.seq_to_uid.len() + data.len(), 0);

        for (seq, items) in data {
            let envelope = Envelope::new(seq, items);
            debug!("added envelope: {envelope:?}");
            let uid = envelope.uid;
            let seq = envelope.seq as usize - 1;
            self.by_uid.insert(uid, envelope);
            self.seq_to_uid[seq] = uid;
        }

        self.imap.replace(imap);
        Ok(())
    }

    fn apply_expunge(&mut self, seq: NonZeroU32) {
        let uid = self.seq_to_uid.remove(seq.get() as usize - 1);
        let envelope = self.by_uid.remove(&uid);
        debug!("removed envelope: {envelope:?}");
    }

    fn apply_fetch(&mut self, seq: NonZeroU32, items: Vec1<MessageDataItem>) {
        for item in items {
            match item {
                MessageDataItem::Flags(flags) => {
                    let Some(uid) = self.seq_to_uid.get(seq.get() as usize - 1) else {
                        debug!("cannot find cached envelope at seq {seq}, ignoring");
                        continue;
                    };

                    let Some(envelope) = self.by_uid.get_mut(uid) else {
                        debug!("cannot find cached envelope at uid {uid}, ignoring");
                        continue;
                    };

                    envelope.flags = flags
                        .into_iter()
                        .map(|f| match f {
                            FlagFetch::Flag(f) => format!("{f}"),
                            FlagFetch::Recent => "\\Recent".into(),
                        })
                        .collect();

                    debug!("updated envelope flags: {envelope:?}");
                }
                item => {
                    debug!("unused message data item: {item:?}, ignoring");
                    continue;
                }
            }
        }
    }
}

/// Decode RFC 2047 MIME-encoded string, falling back to original on error.
pub fn decode_mime(s: &str) -> String {
    let decoder = Decoder::new().too_long_encoded_word_strategy(RecoverStrategy::Decode);
    match decoder.decode(s.as_bytes()) {
        Ok(s) => s,
        Err(err) => {
            debug!("cannot decode rfc2047 string `{s}`: {err}");
            s.to_string()
        }
    }
}

/// Format email address from mailbox and host parts.
fn format_email(addr: &Address<'_>) -> String {
    let mailbox = addr
        .mailbox
        .0
        .as_ref()
        .map(|m| String::from_utf8_lossy(m.as_ref()).to_string())
        .unwrap_or_default();
    let host = addr
        .host
        .0
        .as_ref()
        .map(|h| String::from_utf8_lossy(h.as_ref()).to_string())
        .unwrap_or_default();

    if !mailbox.is_empty() && !host.is_empty() {
        format!("{mailbox}@{host}")
    } else {
        mailbox
    }
}

/// Short format for list view (name OR email, not both).
pub fn format_address_short(addr: &Address<'_>) -> String {
    // If name exists, show decoded name only
    if let Some(n) = &addr.name.0 {
        let name = decode_mime(&String::from_utf8_lossy(n.as_ref()));
        if !name.is_empty() {
            return name;
        }
    }
    // Otherwise show email
    format_email(addr)
}

/// Short addresses formatter for list view.
pub fn format_addresses_short(addrs: &[Address<'_>]) -> String {
    addrs
        .iter()
        .map(format_address_short)
        .collect::<Vec<_>>()
        .join(", ")
}
