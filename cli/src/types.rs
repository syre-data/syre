#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ResourcePathType {
    Absolute,
    Relative,
    Root,
}
