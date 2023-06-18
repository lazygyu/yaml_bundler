use clap::Parser;

#[derive(Parser, Debug)]
pub struct CLIArguments {
    pub input_file: String,
    #[arg(short = 'o', long = "out", default_value = "./aidmat_api_doc.yaml")]
    pub output_file: std::path::PathBuf,

    #[arg(short, long)]
    pub verbose: bool,
}

pub fn parse() -> CLIArguments {
    let args = CLIArguments::parse();
    args
}
