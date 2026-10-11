use anyhow::Result;
use vergen::{Build, Emitter};

pub fn main() -> Result<()> {
    let timestamp_builder = Build::builder().build_timestamp(true).build();

    Emitter::new().add_instructions(&timestamp_builder)?.emit()
}
