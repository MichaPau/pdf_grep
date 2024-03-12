#![allow(dead_code)]
//#![allow(unused_imports)]

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use std::env;

use std::result::Result;

//use grep_utils::Settings;
use rayon::prelude::*;
use clap::{Parser, Subcommand};
//use termcolor::{StandardStream, ColorChoice};
//use walkdir::WalkDir;



// mod xpdf_tools;
mod grep_utils;
mod utils;
// mod parse_utils;
// mod types;

//use xpdf_tools::xpdf_text;
use xpdf_tools::types::XpdfArgs;
use xpdf_tools::XpdfTools;
use xpdf_tools::{self};




#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, group = "input")]
    directory: Option<PathBuf>,
    #[arg(short, long, group = "input")]
    file: Option<PathBuf>,

    // #[arg(short, long)]
    // search: Option<String>,

    #[command(subcommand)]
    command: Actions,
}

#[derive(Debug, Subcommand, Clone)]
enum Actions {
    Search { pattern: String },
    Info,
    Test,
    Text,

}

type BoxError = std::boxed::Box<dyn
	std::error::Error   // must implement Error to satisfy ?
	+ std::marker::Send // needed for threads
	+ std::marker::Sync // needed for threads
>;

fn _write_to_file(file_path: &str, content: &String) {
    let p = Path::new(file_path);
    let mut file = std::fs::File::create(p).unwrap();
    file.write_all(content.as_bytes()).unwrap();

}
fn get_info_file(file_path: &Path) {
    
    //let tools = XpdfTools::builder().extra_args(vec!["-rawdates".to_string(), "-meta".to_string()]).build();
    let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
        .extra_args(vec![XpdfArgs::RawDates, XpdfArgs::Metadata])
        .build();
    
    match tools.pdf_info(file_path) {
        Ok(pdf_info) => {
            println!("{:#?}", pdf_info.info_map);
        },
        Err(e) => println!("{:?}", e),
    }
    
}

fn get_info_dir(dir_path: &Path) {
    println!("info for: {:?}", dir_path);
}

fn search_invoke(dir: &str, list: &Vec<String>, pattern: &String, tools: &XpdfTools) {
    // let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
    // .build();

    let mut settings = grep_utils::Settings::default();

    //let mut output = std::io::stdout();
    //writeln!(settings.stream, "Searching folder {dir}").unwrap();
    for file in list {
        let p = Path::new(file);
        match tools.pdf_text(&p) {
            Ok(content) => {
                grep_utils::search_file(&content, pattern, &mut settings, &p).unwrap();
            },
            Err(e) => {
                io::stderr().write(e.message.as_bytes()).unwrap();
                io::stderr().write(b"\n").unwrap();
                io::stderr().write(e.process_message.as_bytes()).unwrap();
                io::stderr().flush().unwrap();
            },
        }
    }
}
fn search_dir(dir_path: &Path, pattern: &String) {
   
    let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
    .build();
    //println!("{dir_path:?} - {pattern}");
    let pdf_map = utils::get_folder_tree(dir_path);

    

    pdf_map.par_iter().for_each(|(dir, list)| {
        search_invoke(dir, list, pattern, &tools);
        
    });
    //utils::_dump(&pdf_map);

}

fn search_file(file_path: &Path, pattern: &String) {
    //let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut settings = grep_utils::Settings::default();
    let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
        .extra_args(vec![XpdfArgs::Encoding("UTF-8".into())])
        .build();

    match tools.pdf_text(file_path) {
        Ok(content) => {
            grep_utils::search_file(&content, pattern, &mut settings, file_path).unwrap();
        },
        Err(e) => {
            io::stderr().write(e.message.as_bytes()).unwrap();
            io::stderr().write(b"\n").unwrap();
            io::stderr().write(e.process_message.as_bytes()).unwrap();
            io::stderr().flush().unwrap();
        },
    }
    // if let Ok(content) = tools.pdf_text(file_path) {
    //     grep_utils::search_file(&content, pattern);
    // } else {
    //     eprintln!("Error opening pdf file: ")
    // }
    //todo!();
}

fn main() -> Result<(), BoxError>{
    env::set_var("RUST_BACKTRACE", "1");
    println!("{}", xpdf_tools::get_version());
    let cli = Cli::parse();

    match cli.command { 
        Actions::Info => { 
            if let Some(dir) = cli.directory.as_deref() {
                get_info_dir(dir);
            } else if let Some(file) = cli.file.as_deref() {
                get_info_file(&file);
            }
        },
        Actions::Test => { println!("Action: test");},
        Actions::Search {ref pattern}=> {
            if let Some(dir) = cli.directory.as_deref() {
                search_dir(dir, pattern);
            } else if let Some(file) = cli.file.as_deref() {
                search_file(&file, pattern);
            } 
            //println!("Action: search:{}", *pattern);
        },
        Actions::Text => {
            if let Some(_dir) = cli.directory.as_deref() {
                println!("No implemented");
            } else if let Some(file) = cli.file.as_deref() {
                let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
                    .extra_args(vec![XpdfArgs::Simple])
                    .build();
                match tools.pdf_text_as_string(file) {
                    Ok(content) => {
                        println!("{:?}", content);
                        //io::stdout().write_all(content.as_bytes()).unwrap();
                    },
                    Err(e) => {
                        io::stderr().write(e.message.as_bytes()).unwrap();
                        io::stderr().write(b"\n").unwrap();
                        io::stderr().write(e.process_message.as_bytes()).unwrap();
                        io::stderr().flush().unwrap();
                    },
                }
            } 
        }

    }
    println!("{:?}", cli);

    // let search_term = if let Some(search) = cli.search.as_deref() {
    //     search
    // } else {
    //     "nothing"
    // };

    // if let Some(directory) = cli.directory.as_deref() {
    //     println!("Value for dir: {:?}", directory);
    //     for entry in WalkDir::new(directory)
    //         .into_iter()
    //         .filter_map(|r| r.ok())
    //         .filter_map(|file| {
    //             if file.path().extension().map_or(false, |ext| ext == "pdf") {
    //                 Some(file)
    //             } else {
    //                 None
    //             }
    //         }) {
    //             if let Ok(file_result)  = xpdf_tools::pdf_to_binary(entry.path().to_str().unwrap()) {
    //                  let search_result = grep_utils::search_to_console(search_term, &file_result);
    //                  match search_result {
    //                     Ok(()) => {
    //                         println!("all good..");
    //                     },
    //                     Err(e) => println!("{:?}", e),
    //                 }
    //             }
               
    //             println!("{:?}", entry);
    //         }

    // } else {
    //     println!("No directory argument found");
    // }

    // if let Some(file) = cli.file.as_deref() {
    //     println!("Value for file: {:?}", file);
        
    //     // if let Ok(file_result)  = xpdf_tools::pdf_info(file.to_str().unwrap()) {
    //     //     println!("{}", file_result);
    //     // };

    //     if let Ok(file_result)  = xpdf_tools::pdf_to_binary(file.to_str().unwrap()) {
    //         //let search_result = grep_utils::search_to_console(search_term, &file_result);
    //         let search_result = grep_utils::search_to_vec(search_term, &file_result);
    //         match search_result {
    //             Ok(result) => {
    //                 println!("all good..");
    //                 print_result(&result);

    //             },
    //             Err(e) => println!("{:?}", e),
    //         }
    //     };
         
    // } else {
    //     println!("No file argument found");
    // }
   Ok(())
    
}
fn print_result(result: &[(u64, String)]) {
    for item in result {
        println!("{:?}", item);
    }
}

#[ignore]
#[test]
fn test_regexp_bom() {
    //let hay = "<?xpacket begin=\"*\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>";
    //let hay = "n<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?><?xpacket end=\"w\"?>";

    let hay = include_str!("../data/test.txt");
    //let r_str = r"(?-u)<\?xpacket begin=[.\s\u{FEFF}]*";
    let r_str = r"(?s)<\?xpacket begin=.*<?xpacket end=.*>";
    let meta_re = regex::Regex::new(r_str).unwrap();

    let m = meta_re.find(hay);
    if let Some(meta_match) = m {
        println!("{:?}", meta_match.as_str());
    } else {
        println!("no match found");
    }

    assert!(m.is_some());
}

#[ignore]
#[test]
fn normalize_whitespace() {
    use std::io::stdout;
    let hay = include_str!("../data/test.txt");
    let result = hay.lines().filter_map(|l| {
      
        if l.trim_end() != "" {
            Some(l)
        } else {
            None
        }
    }).collect::<Vec<_>>().join("\n");
    //println!("{:?}", result);
    stdout().write_all(result.as_bytes()).unwrap();
    
    // let r = regex::Regex::new(r"(\s\s[ ^\t]+?){2,}+").unwrap();
    // let matches:Vec<_> = r.find_iter(hay).map(|c| c.as_str()).collect();
    // println!("Matches: {:?}", matches);

    // let r_str = r.replace_all(hay, "$1");
    // println!("{}", r_str);


}
// #[allow(dead_code)]
// fn search(what: &str, from: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//     //let s = format!(r"(?i).{{10}}{}.{{10}}", what);
//     let s = format!(r"(?i){}", what);
//     println!("search for: {}", s);
//     let matcher = RegexMatcher::new(s.as_str())?;
//     //let mut matches: Vec<(u64, String)> = vec![];

//     // let spec_match: UserColorSpec = "match:fg:0xff,0xc5,0x12".parse().unwrap();
//     // let spec_match2: UserColorSpec = "match:style:underline".parse().unwrap();
//     // let spec_line: UserColorSpec = "line:fg:0x04,0x85,0xd1".parse().unwrap();
//     //let color_spec = user_spec.to_color_spec();
//     //let specs = ColorSpecs::new(&[spec_line, spec_match, spec_match2]);
//     let specs = grep_utils::build_color_specs();
//     let mut printer = StandardBuilder::new()
//         .color_specs(specs)
//         .build(cli::stdout(if std::io::stdout().is_terminal() {
//             ColorChoice::Auto
//         } else {
//             ColorChoice::Never
//         }));

//     let mut searcher = SearcherBuilder::new()
//         .binary_detection(BinaryDetection::quit(b'\x00'))
//         .multi_line(false)
//         .line_number(true)
//         .build();

//     // searcher.search_slice(&matcher, from,  UTF8(|lnum, line| {
//     //     print!("{}:{}", lnum, line);
//     //     Ok(true)
//     // }),)?;
//     searcher.search_slice(&matcher, from,  printer.sink(&matcher),)?;
//     // Searcher::new().search_slice(&matcher, from, UTF8(|lnum, line| {
//     //     // We are guaranteed to find a match, so the unwrap is OK.
//     //     let mymatch = matcher.find(line.as_bytes())?.unwrap();
//     //     matches.push((lnum, line[mymatch].to_string()));
//     //     Ok(true)
//     // }))?;

//     // for ma in matches {
//     //     println!("{:?}", ma);
//     // }
//     Ok(())
// }
#[test]
fn test_encoding() {
    let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
        .extra_args(vec![XpdfArgs::Encoding("UTF-8".into())])
        .build();
        //let result = tools.pdf_text_as_string(Path::new("./data/descartes_meditations.pdf")).unwrap();
        let result = tools.pdf_text_as_string(Path::new("./data/sample_text.pdf")).unwrap();

        println!("{}", result);
        io::stdout().write_all(result.as_bytes()).unwrap();
        let _w = _write_to_file("./data/test_02.txt", &result);

}

