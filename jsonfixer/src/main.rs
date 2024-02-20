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

    let mut binding = ParsedDocument::from_string(content, None);
    let parsed_document = binding.as_mut().unwrap();

    let bytes = format.to_utf8(parsed_document)?;
    let jsonstr = std::str::from_utf8(&bytes).unwrap();
    print!("{}", jsonstr);

    let mut json5_parser = tree_sitter::Parser::new();
    json5_parser
        .set_language(tree_sitter_json5::language())
        .expect("Error loading Rust grammar");
    let tree = json5_parser.parse(jsonstr, None).unwrap();
    println!("{}", tree.root_node().to_sexp());
    let querystr = r#"
        (member name:  (identifier)* @field-name)
        (member name:  (identifier) value: (string)* @field-str-value)
    "#;
    let text_provider: &[u8] = &[];
    let query = tree_sitter::Query::new(tree_sitter_json5::language(), querystr).unwrap();
    let mut new_json_bytes: Vec<u8> = Vec::new();
    let mut cursor = tree_sitter::QueryCursor::new();
    let captures = cursor.captures(&query, tree.root_node(), text_provider);
    let mut low = 0;
    for query_capture_tuple in captures {
        let query_match = query_capture_tuple.0;
        let node_type = query_match.pattern_index;
        let capture = query_match.captures.first().unwrap();

        let range = capture.node.byte_range();
        // println!("{:?}", range);

        new_json_bytes.extend(&bytes[low..range.start]);
        low = range.start;
        if node_type == 0 {
            new_json_bytes.push(b'"');
            new_json_bytes.extend(&bytes[low..range.end]);
            low = range.end;
            new_json_bytes.push(b'"');
        } else if node_type == 1 {
            new_json_bytes.extend(&bytes[low..range.end]);
            low = range.end;
        }
    }
    new_json_bytes.extend(&bytes[low..bytes.len()]);
    println!("{}", std::str::from_utf8(&new_json_bytes).unwrap());
    Ok(())
}
