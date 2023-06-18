mod cli_parser;
mod logger;
mod yaml_process;

use std::path::Path;

fn main() {
    let args = cli_parser::parse();

    let base_path = Path::new(&args.input_file).parent().unwrap();
    let log = logger::Logger {
        verbose: args.verbose,
    };

    log.println(
        true,
        format!(
            "input file  : {} \n output file : {} \n base path   : {} \n",
            args.input_file.as_str(),
            args.output_file.display(),
            base_path.display(),
        )
        .as_str(),
    );

    let result = yaml_process::process(&args.input_file, &base_path, &log);
    match result {
        Err(e) => {
            eprintln!("Error: {}", e);
        }
        Ok(yaml_string) => {
            log.println(true, "Writing to a file");
            std::fs::write(args.output_file, yaml_string)
                .expect("An error occurs while writing the file");
            log.println(true, "All done");
        }
    }
}
