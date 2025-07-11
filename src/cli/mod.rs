use std::{collections::HashMap, env::Args, fs};

use format::{FileFormat, SUPPORTED_FORMATS};

mod format;

#[derive(Clone, Debug)]
struct ArgumentParseError(String);

pub fn parse_args() {
    let args = std::env::args();
    let parsed_args = get_arg_flags(args).unwrap();

    match parsed_args {
        GimgArguments::Help(subject) => print_help(subject),
        GimgArguments::Version => print_version(),
        GimgArguments::Command {
            input_file,
            output_file,
            keyword_args: _,
        } => {
            let input_format = determine_file_format_by_name(&input_file)
                .unwrap_or_else(|| panic!("Unrecognized input file format {input_file}"));
            let output_format = determine_file_format_by_name(&output_file)
                .unwrap_or_else(|| panic!("Unrecognized input file format {input_file}"));
        }
    }
}

/**
 * gimg input.ppm out.png
 * gimg input.ppm --out "interlace=true;bit_depth=10" out.png
 */
fn get_arg_flags(mut args: Args) -> Result<GimgArguments, ArgumentParseError> {
    let mut unprefixed_args: Vec<String> = Vec::new();
    let mut keyword_args: HashMap<String, String> = HashMap::new();

    let _current_dir = args.next();

    if args.len() == 0 {
        return Ok(GimgArguments::Help("".to_string()));
    }

    while let Some(mut arg) = args.next() {
        let (flag, value) = if arg.starts_with("--") {
            let flag = arg.split_off(2);
            let value = args.next();

            (flag, value)
        } else if arg.starts_with("-") {
            let (flag, value) = arg.split_at(1);
            let value = Some(value.to_string()).filter(|s| s.is_empty());

            (flag.to_string(), value)
        } else {
            unprefixed_args.push(arg);
            continue;
        };

        let value = match value {
            Some(value) => value,
            None => {
                if flag == "help" || flag == "h" || flag == "version" || flag == "v" {
                    "".to_string()
                } else {
                    return Err(ArgumentParseError(format!("{flag} requires an argument")));
                }
            }
        };

        keyword_args.insert(flag, value);
    }

    if let Some(subject) = keyword_args.get("help").or(keyword_args.get("h")) {
        return Ok(GimgArguments::Help(subject.to_string()));
    }

    if let Some(_subject) = keyword_args.get("version").or(keyword_args.get("v")) {
        return Ok(GimgArguments::Version);
    }

    if unprefixed_args.len() != 2 {
        return Err(ArgumentParseError(format!(
            "Expected exactly 2 unprefixed arguments, input and output, found {} instead",
            unprefixed_args.len()
        )));
    }

    let [input_file, output_file]: [String; 2] = unprefixed_args.try_into().unwrap();
    Ok(GimgArguments::Command {
        input_file,
        output_file,
        keyword_args,
    })
}

fn print_help(subject: String) {
    println!("[help for {subject}]");
}

fn print_version() {
    println!("0.0.1");
}

fn determine_file_format_by_name(filename: &str) -> Option<FileFormat> {
    match fs::read(filename) {
        Ok(file_bytes) => {
            for format in SUPPORTED_FORMATS {
                if format.is_format_by_signature(&file_bytes) {
                    return Some(format);
                }
            }

            for format in SUPPORTED_FORMATS {
                if format.is_format_by_extension(filename) {
                    return Some(format);
                }
            }

            None
        }
        Err(e) => panic!("error reading input file {filename}: {e}"),
    }
}

#[derive(Debug)]
enum GimgArguments {
    Help(String),
    Version,
    Command {
        input_file: String,
        output_file: String,
        keyword_args: HashMap<String, String>,
    },
}
