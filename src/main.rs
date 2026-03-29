mod app;
mod categories;
mod cleanup;
mod cli;
mod context;
mod detectors;
mod fs_utils;
mod scanner;
mod types;

use anyhow::Result;

fn main() -> Result<()> {
    app::run()
}
