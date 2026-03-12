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

mod cli;
mod config;
#[cfg(feature = "imap")]
mod imap;

use clap::Parser;
use pimalaya_toolbox::terminal::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

use crate::cli::MiradorCli;

fn main() {
    let cli = MiradorCli::parse();

    Logger::init(&cli.log);

    let mut printer = StdoutPrinter::new(&cli.json);
    let config_paths = cli.config_paths.as_ref();
    let account_name = cli.account.name.as_deref();

    let result = cli.command.exec(&mut printer, config_paths, account_name);

    ErrorReport::eval(&mut printer, result)
}
