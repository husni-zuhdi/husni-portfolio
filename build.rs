use anyhow::Result;
use vergen_gitcl::{BuildBuilder, Emitter};

pub fn main() -> Result<()> {
    let timestamp_builder = BuildBuilder::default().build_timestamp(true).build()?;

    Emitter::default()
        .add_instructions(&timestamp_builder)?
        .emit()
}
