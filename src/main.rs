use comrak::{Arena, parse_document, ComrakOptions, nodes::{AstNode, NodeValue}};
use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
    process::Command,
    str,
};
use walkdir::WalkDir;

enum Opts {
    Build {
        args: Vec<String>
    },
    Check {
        args: Vec<String>
    },
    Run {
        args: Vec<String>
    },
    Doc {
        args: Vec<String>
    },
    Test {
        args: Vec<String>
    },
    Bench {
        args: Vec<String>
    },
}

impl Opts {
    fn from_args() -> Result<Self, Box<dyn Error>> {
        let mut args = env::args();
        args.next();
        use self::Opts::*;
        match args.next().unwrap_or(String::new()).as_str() {
            "build" =>{
                Ok(Build { args: args.collect() })
            },
            "check" =>{
                Ok(Check { args: args.collect() })
            },
            "run" =>{
                Ok(Run { args: args.collect() })
            },
            "doc" =>{
                Ok(Doc { args: args.collect() })
            },
            "test" =>{
                Ok(Test { args: args.collect() })
            },
            "bench" =>{
                Ok(Bench { args: args.collect() })
            },
            "" => Err("You need to give cargo-lit a command".into()),
            c => Err(format!("'{}' is an unsupported command", c).as_str().into()),
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args()?;
    let src_path = manifest_dir()?.join("src");
    for entry in WalkDir::new(src_path).contents_first(true).into_iter().filter_entry(|e| {
        let file = {
            match e.metadata() {
                Err(_) => false,
                Ok(m) => m.is_file(),
            }
        };
        let md = e.file_name().to_str().map(|s| s.ends_with(".md")).unwrap_or(false);
        file && md
    })
    {
        let entry = entry?;
        let mut output_path = PathBuf::from(entry.path());
        output_path.set_extension("rs");
        let arena = Arena::new();
        let root = parse_document(
            &arena,
            &fs::read_to_string(entry.path())?,
            &ComrakOptions::default()
        );

        fs::write(&output_path, markdown_to_rust(root)?)?;


    }
    env::set_current_dir(&manifest_dir()?)?;
    let mut command = Command::new("cargo");
    match opts {
        Opts::Bench { args } => {
            command.arg("bench");
            if !args.is_empty() {
                command.args(&args);
            }
        },
        Opts::Build { args } => {
            command.arg("build");
            if !args.is_empty() {
                command.args(&args);
            }
        },
        Opts::Check { args } => {
            command.arg("check");
            if !args.is_empty() {
                command.args(&args);
            }
        },
        Opts::Doc { args } => {
            command.arg("doc");
            if !args.is_empty() {
                command.args(&args);
            }
        },
        Opts::Run { args } => {
            command.arg("run");
            if !args.is_empty() {
                command.args(&args);
            }
        },
        Opts::Test { args } => {
            command.arg("test");
            if !args.is_empty() {
                command.args(&args);
            }
        },
    }
    command.spawn()?;
    Ok(())
}

fn manifest_dir() -> Result<PathBuf, Box<dyn Error>> {
    for path in env::current_dir()?.ancestors() {
        let manifest_path = path.join("Cargo.toml");
        if manifest_path.exists() {
           return Ok(path.into());
        }
    }

    Err("Unable to find a crate manifest. Are you in a cargo project at all?".into())
}

fn markdown_to_rust<'a>(input: &'a AstNode<'a>) -> Result<String, Box<dyn Error>> {
    let mut output = String::new();

    let mut traverse = input.descendants().peekable();
    while let Some(node) = traverse.next() {
        match node.data.borrow().value {
            NodeValue::BlockQuote => unimplemented!(),
            NodeValue::Code(ref _code) => unimplemented!(),
            NodeValue::CodeBlock(ref block) => {
                output.push('\n');
                output.push_str(str::from_utf8(&block.literal)?);
            },
            NodeValue::DescriptionDetails => unimplemented!(),
            NodeValue::DescriptionItem(ref _item) => unimplemented!(),
            NodeValue::DescriptionList => unimplemented!(),
            NodeValue::DescriptionTerm => unimplemented!(),
            // We want to skip this as it's the top level enum variant
            NodeValue::Document => (),
            NodeValue::Emph => unimplemented!(),
            NodeValue::FootnoteDefinition(ref _footnote) => unimplemented!(),
            NodeValue::FootnoteReference(ref _footnote) => unimplemented!(),
            NodeValue::Heading(ref heading) => {
                output.push_str("// ");
                for _ in 0..heading.level {
                    output.push('#');
                }
                output.push(' ');
                if let Some(node) = traverse.peek() {
                    if let NodeValue::Text(ref text) = node.data.borrow().value {
                        output.push_str(str::from_utf8(&text)?);
                        output.push('\n');
                        traverse.next();
                    }
                }
            },
            NodeValue::HtmlBlock(ref _block) => unimplemented!(),
            NodeValue::HtmlInline(ref _inline) => unimplemented!(),
            NodeValue::Image(ref _image) => unimplemented!(),
            NodeValue::Item(ref _item) => unimplemented!(),
            NodeValue::LineBreak => {
                output.push('\n');
                if let Some(node) = traverse.peek() {
                    if let NodeValue::Text(_) = node.data.borrow().value {
                        output.push_str("// ");
                    }
                }
            },
            NodeValue::Link(ref _link) => unimplemented!(),
            NodeValue::List(ref _list) => unimplemented!(),
            NodeValue::Paragraph => {
                output.push_str("// ");
            },
            NodeValue::SoftBreak => {
                output.push('\n');
                if let Some(node) = traverse.peek() {
                    if let NodeValue::Text(_) = node.data.borrow().value {
                        output.push_str("// ");
                    }
                }
            },
            NodeValue::Strikethrough => unimplemented!(),
            NodeValue::Strong => unimplemented!(),
            NodeValue::Superscript => unimplemented!(),
            NodeValue::Table(ref _table) => unimplemented!(),
            NodeValue::TableCell => unimplemented!(),
            NodeValue::TableRow(ref _row) => unimplemented!(),
            NodeValue::TaskItem(ref _item) => unimplemented!(),
            NodeValue::Text(ref text) => {
                output.push_str(str::from_utf8(&text)?)
            },
            NodeValue::ThematicBreak => unimplemented!(),
        }
    }

    Ok(output)
}
