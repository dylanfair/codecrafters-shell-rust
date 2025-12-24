use crate::input::utils::RedirectOptions;

#[derive(Clone, Debug)]
pub struct InputBlock {
    pub command: String,
    pub args: Vec<String>,
    pub redirect_options: RedirectOptions,
    pub piped: bool,
}

impl InputBlock {
    pub fn new(
        command: String,
        args: Vec<String>,
        redirect_options: RedirectOptions,
        piped: bool,
    ) -> InputBlock {
        InputBlock {
            command,
            args,
            redirect_options,
            piped,
        }
    }
}
