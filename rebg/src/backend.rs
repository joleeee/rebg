/// - Gives the specific tracer to be ran, with options
/// - Parses output
trait Backend {
    fn command(executable: Path, arch: Arch) -> (String, Vec<String>);
}


// we use argh so we can easily change "arguments"
#[derive(argh)]
struct QEMU {
}

impl Backend for QEMU {
    fn command(executable: Path, arch: Arch) -> (String, Vec<String>) {
        let binary_ext = match arch {
            _ => todo!()
        };
        let options = todo!();

        (binary, options)
    }
}