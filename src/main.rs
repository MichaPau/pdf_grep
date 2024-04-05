//#![allow(dead_code)]
//#![allow(unused_imports)]

//use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use std::env;

use std::result::Result;

// mod xpdf_tools;
mod grep_utils;
// mod grep_utils2;
mod utils;
mod settings;
mod pdf_tools;

// mod parse_utils;
// mod types;

use pdf_tools::XpdfWrapper;


use xpdf_tools::types::XpdfArgs;
//use xpdf_tools::xpdf_info::PdfInfo;
use xpdf_tools::XpdfTools;
use xpdf_tools::{self};

use crate::pdf_tools::{AvailablePdfTools, PdfDummyTool};
use crate::settings::{Actions, Settings, ShortenLineMode};

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

fn main() -> Result<(), BoxError>{
    env::set_var("RUST_BACKTRACE", "1");
    println!("{}", xpdf_tools::get_version());

    //let mut settings = Settings::default();
    let mut settings = Settings::new();
    settings.shorten_line_mode = ShortenLineMode::Trim(25);
    settings.tools = match settings.use_pdf_tool {
        AvailablePdfTools::UseXpdfTools => {
            let t = XpdfTools::builder(PathBuf::from(settings.xpdf_tools_folder.as_ref().unwrap())).unwrap()
                    .extra_args(vec![XpdfArgs::Encoding("UTF-8".into())])
                    .build();
            Box::new(XpdfWrapper {tools: t})
        },
        AvailablePdfTools::UsePdfDummyTool => Box::new(PdfDummyTool {}),
    };
    // let _tools = XpdfTools::builder(PathBuf::from("./tools/xpdf-tools-win-4.05/bin64/")).unwrap()
    //                 .extra_args(vec![XpdfArgs::Encoding("UTF-8".into())])
    //                 .build();    

    // settings.tools = Box::new(XpdfWrapper {tools: _tools});
    // let cli = Cli::parse();
    let cli = settings.cli.as_ref().unwrap();
    match cli.command { 
        Actions::Info => { 
            if let Some(dir) = cli.directory.as_deref() {
                pdf_tools::get_info_dir(dir);
            } else if let Some(file) = cli.file.as_deref() {
                pdf_tools::get_info_file(file, &settings);
            }
        },
        Actions::Test => { println!("Action: test");},
        Actions::Search {ref pattern}=> {
            if let Some(dir) = cli.directory.as_deref() {
                pdf_tools::search_dir(dir, pattern, &settings)?;
            } else if let Some(file) = cli.file.as_deref() {
                // pdf_tools::search_file(&file, pattern, &settings);
                settings.tools.search_file(file, pattern, &settings)?;
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
                        io::stderr().write_all(e.message.as_bytes()).unwrap();
                        io::stderr().write_all(b"\n").unwrap();
                        io::stderr().write_all(e.process_message.as_bytes()).unwrap();
                        io::stderr().flush().unwrap();
                    },
                }
            } 
        },
        Actions::Rand {ref length} => {
            let mut snippet_length = 150;
            if let Some(l) = length {
                snippet_length = *l;
            }
            if let Some(dir) = cli.directory.as_deref() {
                let result = pdf_tools::get_random_text(dir, &settings, snippet_length)?;
                println!("{}", result.0);
                println!("{}", result.1);
            }
        }

    }
    println!("{:?}", cli);

    
   Ok(())
    
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

#[ignore]
#[test]
fn test_folder_mode() {
    use utils::get_folder_files;
    use settings::FolderSearchMode;
    use settings::ShortenLineMode;
    use std::time::Instant;
    use termcolor::ColorChoice;

    use rayon::prelude::*;

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
        //pdf_tools::search_invoke_file(String::from(file), &pattern, &_settings);
       _settings.tools.search_file(Path::new(&file), &pattern, &_settings).unwrap();
    });

    let elapsed2 = start.elapsed();
    println!("Duration folder:{:?}", elapsed1.as_millis());
    println!("Duration files:{:?}", elapsed2.as_millis());
   
}


