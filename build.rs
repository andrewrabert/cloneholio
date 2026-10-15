use vergen_gix::{Emitter, Gix};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gix = Gix::builder().sha(true).dirty(true).build();
    Emitter::default().add_instructions(&gix)?.emit_and_set()?;
    println!("cargo::rustc-env=VERSION_SUFFIX={}", version_suffix());
    Ok(())
}

fn version_suffix() -> String {
    match git_suffix() {
        Some(suffix) if !env!("CARGO_PKG_VERSION_PRE").is_empty() => suffix,
        _ => String::new(),
    }
}

fn git_suffix() -> Option<String> {
    let hash = std::env::var("VERGEN_GIT_SHA").ok()?;
    let dirty = match std::env::var("VERGEN_GIT_DIRTY").as_deref() {
        Ok("true") => "-dirty",
        _ => "",
    };
    Some(format!("+{hash}{dirty}"))
}
