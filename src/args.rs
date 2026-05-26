#[derive(Clone, Copy, Default, clap::ValueEnum)]
pub enum Backend {
    #[cfg(feature = "backend-dummy")]
    Dummy,
    #[cfg(feature = "backend-glfw")]
    Glfw,
    #[default]
    Auto,
}

#[derive(clap::Parser)]
pub struct Args {
    #[clap(short, long, value_enum, default_value_t = Backend::Auto)]
    pub backend: Backend,
}
