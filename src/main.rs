#![allow(dead_code)]
//#![allow(unused_imports)]

//use std::collections::BTreeMap;
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
mod settings;

// mod parse_utils;
// mod types;

use settings::XpdfWrapper;

//use xpdf_tools::xpdf_text;
use xpdf_tools::types::XpdfArgs;
//use xpdf_tools::xpdf_info::PdfInfo;
use xpdf_tools::{PdfError, XpdfTools};
use xpdf_tools::{self};

use crate::settings::{PdfTestTools, Settings};
//use crate::utils::get_folder_files;




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

//tools: &dyn PDFTools
fn search_invoke_file(file: String, pattern: &String, settings: &Settings) {
    //let settings = settings::Settings::default();

    //let mut output = std::io::stdout();
    //writeln!(settings.stream, "Searching folder {dir}").unwrap();
   
        let p = Path::new(&file);
        match settings.tools.pdf_text(&p) {
            Ok(content) => {
                grep_utils::search_file(&content, pattern, &settings, &p).unwrap();
            },
            Err(e) => {
                if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
                    io::stderr().write(pdf_error.message.as_bytes()).unwrap();
                    io::stderr().write(b"\n").unwrap();
                    io::stderr().write(pdf_error.process_message.as_bytes()).unwrap();
                    
                } else {
                    io::stderr().write(e.to_string().as_bytes()).unwrap();
                }
                io::stderr().flush().unwrap();
            },
        }
    
}
//tools: &dyn PDFTools
fn search_invoke_folders(_dir: &str, list: &Vec<String>, pattern: &String, settings: &Settings) {
    // let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
    // .build();

    // let mut settings = settings::Settings {
    //     tools: Box::new(XpdfWrapper {tools}), 
    //     ..Default::default()
    // };
    //let settings = settings::Settings::default();

    //let mut output = std::io::stdout();
    //writeln!(settings.stream, "Searching folder {dir}").unwrap();
    for file in list {
        let p = Path::new(file);
        match settings.tools.pdf_text(&p) {
            Ok(content) => {
                grep_utils::search_file(&content, pattern, &settings, &p).unwrap();
            },
            Err(e) => {
                if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
                    io::stderr().write(pdf_error.message.as_bytes()).unwrap();
                    io::stderr().write(b"\n").unwrap();
                    io::stderr().write(pdf_error.process_message.as_bytes()).unwrap();
                    
                } else {
                    io::stderr().write(e.to_string().as_bytes()).unwrap();
                }
                io::stderr().flush().unwrap();
            },
        }
    }
}
fn search_dir(dir_path: &Path, pattern: &String, settings: &mut Settings) {
   
    // let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
    // .build();
    //println!("{dir_path:?} - {pattern}");
    let pdf_map = utils::get_folder_tree(dir_path);

    pdf_map.par_iter().for_each(|(dir, list)| {
        //search_invoke(dir, list, pattern, &tools);
        search_invoke_folders(dir, list, pattern, &settings);
        //search_invoke(dir, list, pattern, settings);
        
    });
    //utils::_dump(&pdf_map);

}

fn search_file(file_path: &Path, pattern: &String, settings: &mut Settings) {
    //let mut stdout = StandardStream::stdout(ColorChoice::Always);
    //let mut settings = settings::Settings::default();
    
    // let tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
    //     .extra_args(vec![XpdfArgs::Encoding("UTF-8".into())])
    //     .build();

    match settings.tools.pdf_text(file_path) {
        Ok(content) => {
            grep_utils::search_file(&content, pattern, settings, file_path).unwrap();
        },
        Err(e) => {
            if let Some(pdf_error) = e.downcast_ref::<PdfError>() {
                io::stderr().write(pdf_error.message.as_bytes()).unwrap();
                io::stderr().write(b"\n").unwrap();
                io::stderr().write(pdf_error.process_message.as_bytes()).unwrap();
                
            } else {
                io::stderr().write(e.to_string().as_bytes()).unwrap();
            }
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

    let mut settings = Settings::default();
    let _tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
                    .extra_args(vec![XpdfArgs::Encoding("UTF-8".into())])
                    .build();    

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
                settings.tools = Box::new(XpdfWrapper {tools: _tools});
                search_dir(dir, pattern, &mut settings);
            } else if let Some(file) = cli.file.as_deref() {
                
                //settings.tools = Box::new(XpdfWrapper {tools});
                settings.tools = Box::new(PdfTestTools {});
                search_file(&file, pattern, &mut settings);
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

#[ignore]
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

// #[ignore]
// #[test]
// fn test_pdf_extract() {
//     //use settings::PdfExtractWrapper;

//     let mut _settings = Settings::default();
//     _settings.tools = Box::new(settings::PdfExtractWrapper{});
//     let file = "./data/Dubliners_James_Joyce.pdf";
//     let _pattern = "dublin".to_string();

//     match _settings.tools.pdf_text(Path::new(file)) {
//         Ok(result) => println!("{:?}", String::from_utf8_lossy(&result)),
//         Err(e) => eprintln!("{:?}", Box::new(e)),
//     }
//     //search_file(Path::new(file), &pattern, &mut _settings);


// }

//#[ignore]
#[test]
fn test_folder_mode() {
    use utils::get_folder_files;
    use settings::FolderSearchMode;
    use settings::ShortenLineMode;
    use std::time::Instant;
    use termcolor::ColorChoice;

    let mut _settings = Settings::default();
    let tools_folder = "./tools/xpdf-tools-linux-4.05/bin64/";
    let _tools = XpdfTools::builder(PathBuf::from(tools_folder)).unwrap()
            .extra_args(vec![XpdfArgs::Encoding("UTF-8".into())])
            .build();

    _settings.tools = Box::new(XpdfWrapper{tools: _tools});
    //_settings.tools = Box::new(settings::PdfExtractWrapper{});
    _settings.folder_search_mode = FolderSearchMode::ThreadPerFile;
    _settings.color_choice = ColorChoice::Auto;
    _settings.shorten_line_mode = ShortenLineMode::Trim(10);
    
    //let folder = "c:\\Data\\Library\\Books";
    let folder = "./data/Math";
    let _file_list = get_folder_files(Path::new(folder));
    // let pdf_map = get_folder_tree(Path::new(folder));

    // let _file_list:Vec<_> = pdf_map.iter()
    //     .map(|(_dir, list)| {
    //         let d = _dir;
    //         list.into_iter().map(|file| (d.to_owned(), file)).collect::<Vec<_>>()
    //     })
    //     .flatten()
    //     .collect();

    // for item in file_list {
    //     println!("{:?}", item);
    // }

    let mut start = Instant::now();
    let pattern = "theory".to_string();
    // pdf_map.par_iter().for_each(|(dir, list)| {
    //     //search_invoke(dir, list, pattern, &tools);
    //     search_invoke_folders(dir, list, &pattern, &_settings);
        
    // });
    let elapsed1 = start.elapsed();
    

    start = Instant::now();
    _file_list.par_iter().for_each(|(_dir, file)| {
        search_invoke_file(String::from(file), &pattern, &_settings);
    });

    let elapsed2 = start.elapsed();
    println!("Duration folder:{:?}", elapsed1.as_millis());
    println!("Duration files:{:?}", elapsed2.as_millis());
   
}


