use clap::Parser;
use clap_stdin::FileOrStdin;
use json5format::{FormatOptions, Json5Format, ParsedDocument};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Replace (overwrite) the input file with the formatted result
    #[clap(short, long, default_value_t = false)]
    replace: bool,

    /// Suppress trailing commas (otherwise added by default)
    #[clap(short, long, default_value_t = true)]
    no_trailing_commas: bool,

    /// Objects or arrays with a single child should collapse to a single line; no trailing comma
    #[clap(short, long, default_value_t = false)]
    one_element_lines: bool,

    /// Sort arrays of primitive values (string, number, boolean, or null) lexicographically
    #[clap(short, long, default_value_t = false)]
    sort_arrays: bool,

    #[clap(short, long, default_value_t = 4)]
    indent: usize,

    #[clap(default_value = "-")]
    input: FileOrStdin,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let options = FormatOptions {
        indent_by: args.indent,
        trailing_commas: !args.no_trailing_commas,
        collapse_containers_of_one: args.one_element_lines,
        sort_array_items: args.sort_arrays,
        ..Default::default()
    };
    let format = Json5Format::with_options(options)?;

    let content = args.input.contents()?;

    // println!("input={}", content);
    let mut binding = ParsedDocument::from_string(content, None);
    let parsed_document = binding.as_mut().unwrap();

    let bytes = format.to_utf8(parsed_document)?;
    let jsonstr = std::str::from_utf8(&bytes).unwrap();
    print!("{}", jsonstr);
    Ok(())
}
