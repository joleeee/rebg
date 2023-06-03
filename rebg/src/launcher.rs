/// - Runs the binary
/// - Exposes file read (for used libraries)
trait Launcher {
    type Error;
    fn launch(&self) -> Result<(recv, Launched), Error>;
    fn read_file(&self, path: Path) -> Result<Vec<u8>, Error>;
}

/// Arguments for creation (if spawing new)
struct DockerArgs {
    /// Option existing container
    id: Option<String>,
    
    /// Optional image, ignored if `id` is set
    image: Option<String>,
    
    /// Optional arch override
    arch: Option<Arch>,
}

/// This has the setup image
struct Docker {
    // docker specfic. native can't really do otherwise (except, maybe? like, rosetta stuff???)
    target_arch: Arch,
    
    /// The running container
    id: String,
}

impl Docker {
    fn image_name(&self) -> String {
        match self.image {
            None => {
                format!("rebg:{}", self.target_arch.dockername())
            }
            Some(s) => s,
        }
    }

    fn kill_existing() {
    }

    fn spawn_new() -> String {
    }

    /// Follows all symlinks
    fn get_absolute_path(&self) -> String {

    }
}

impl Launcher for Docker {
    fn read_file(&self, path: Path) -> Vec<u8> {
    }
}