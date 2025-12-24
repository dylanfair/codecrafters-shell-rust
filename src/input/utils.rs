pub fn parse_input(arguments: &str) -> Vec<String> {
    let mut parsed_arguments = vec![];
    let mut single_quotes = false;
    let mut double_quotes = false;
    let mut escape = false;
    let mut word = String::new();
    for char in arguments.chars() {
        match char {
            '\'' => {
                if double_quotes || escape {
                    word.push(char);
                } else {
                    single_quotes = !single_quotes;
                }
            }
            '"' => {
                if escape && double_quotes {
                    word.pop();
                    word.push(char);
                } else if escape || single_quotes {
                    word.push(char);
                } else {
                    double_quotes = !double_quotes;
                }
            }
            '\\' => {
                if escape && double_quotes {
                    word.pop();
                    word.push(char);
                } else if single_quotes {
                    word.push(char);
                } else if double_quotes {
                    word.push(char);
                    escape = !escape;
                    continue;
                } else {
                    escape = !escape;
                    continue;
                }
            }
            ' ' => {
                if single_quotes || double_quotes || escape {
                    word.push(char);
                } else if !word.is_empty() {
                    parsed_arguments.push(word.clone());
                    word = String::new();
                }
            }
            _ => {
                if double_quotes && escape {
                    match char {
                        '$' | '`' | '\n' => {
                            word.pop();
                            word.push(char);
                        }
                        _ => word.push(char),
                    }
                } else {
                    word.push(char);
                }
            }
        }
        escape = false;
    }
    // push in whatever the last word was
    if !word.is_empty() {
        parsed_arguments.push(word.clone());
    }
    parsed_arguments
}
