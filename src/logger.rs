pub struct Logger {
    pub verbose: bool,
}

impl Logger {
    pub fn print(&self, verbose: bool, message: &str) {
        if verbose && !self.verbose {
            return;
        }
        print!("{}", message);
    }

    pub fn println(&self, verbose: bool, message: &str) {
        self.print(verbose, &format!("{}\n", message).as_str());
    }
}
