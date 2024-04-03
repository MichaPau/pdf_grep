use std::io::{self, IsTerminal};
use std::path::PathBuf;

use grep::printer::{ColorSpecs, Standard, StandardBuilder, Summary, SummaryBuilder};
use grep::searcher::{BinaryDetection, Searcher, SearcherBuilder};
use serde::{Deserialize, Serialize};
use termcolor::{BufferedStandardStream, ColorChoice, WriteColor};
use termcolor::{Color, ColorSpec};

use clap::{Parser, Subcommand};
use crate::pdf_tools::{AvailablePdfTools, PDFTools, PdfDummyTool};

use self::toml_settings::TomlSettings;

mod toml_settings;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(short, long, group = "input")]
    pub directory: Option<PathBuf>,
    #[arg(short, long, group = "input")]
    pub file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Actions,

    #[arg(short, long)]
    pub xpdf_path: Option<PathBuf>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Actions {
    Search { pattern: String },
    Info,
    Test,
    Text,

}
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum FolderSearchMode {
    ThreadPerFolder,
    ThreadPerFile,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ShortenLineMode {
    None,
    Trim(usize),
}


#[derive(Debug)]
pub struct Settings {
    // pub stream: StandardStream,
    //pub searcher: Searcher,
    
    //pub color_specs: SearchColorSpecs,
    pub search_color_specs: ColorSpecs,
    pub info_color_spec: ColorSpec,

    pub tools: Box<dyn PDFTools 
        + std::marker::Send // needed for threads
	    + std::marker::Sync
        >,
    pub folder_search_mode: FolderSearchMode,
    pub print_text: bool,
    pub shorten_line_mode: ShortenLineMode,
    pub color_choice: ColorChoice,

    pub cli: Option<Cli>,
    pub use_pdf_tool: AvailablePdfTools,
    pub xpdf_tools_folder: Option<PathBuf>,
    
}

impl Settings {
    pub fn new() -> Self {
        let mut settings = Settings::default();

        Settings::merge_toml_settings(&mut settings);
        if cfg!(not(test)) {
            println!("Parse cli - not testing");
            settings.cli = Some(Cli::parse());
        }
        
        settings
    }

    fn merge_toml_settings(settings: &mut Settings) {

        let toml: TomlSettings;

        match TomlSettings::load() {
            Ok(toml_loaded) => toml = toml_loaded,

            Err(e) => {
                if let Some(_e) = e.downcast_ref::<io::Error>() {
                    toml = TomlSettings::create_default().unwrap();
                } else {
                    panic!("{:?}", e.to_string());
                }
            }
        }
        //ColorSpecs::new(&default_color_specs())
        settings.search_color_specs = ColorSpecs::new(&toml.make_color_specs());
        settings.info_color_spec = toml.get_info_color_spec();
        settings.use_pdf_tool = toml.use_pdf_tool;
        settings.xpdf_tools_folder = toml.xpdf_tools_folder;
        
        // for color_item in toml.colors {
        //     match color_item.name.as_str() {
        //         "match" => { settings.color_specs.match_spec = Settings::create_color_item(&color_item); },
        //         "text" => { settings.color_specs.text_spec = Settings::create_color_item(&color_item); },
        //         "info" => { settings.color_specs.info_spec = Settings::create_color_item(&color_item); },
        //         "extra" => { settings.color_specs.extra_spec = Settings::create_color_item(&color_item); },
        //         _ => (),
        //     }
        // }
    }
    // fn create_color_item(item: &ConfigColorSpec) -> ColorSpec {

    //     let mut spec = ColorSpec::new();
    //     if let Some(fg) = item.fg {
    //         spec.set_fg(Some(Color::Rgb(fg.0, fg.1, fg.2)));
    //     }
    //     if let Some(bg) = item.bg {
    //         spec.set_bg(Some(Color::Rgb(bg.0, bg.1, bg.2)));
    //     }

    //     for (style, flag) in &item.styles {
    //         if *flag == true {
    //             match style.as_str() {
    //                 "bold" => {spec.set_bold(true);},
    //                 "intense" => {spec.set_intense(true);},
    //                 "underline" => {spec.set_underline(true);},
    //                 "dimmed" => {spec.set_dimmed(true);},
    //                 "italic" => {spec.set_italic(true);},
    //                 "reset" => {spec.set_reset(true);},
    //                 "strikethrough" => {spec.set_strikethrough(true);},
    //                 _ => (),
    //             }
    //         }
    //     }

    //     spec
    // }
    //impl std::io::Write + WriteColor
    pub fn create_color_writer(&self) -> impl std::io::Write + WriteColor {
        //let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
        //let stream = StandardStream::stdout(self.color_choice);
        let stream = BufferedStandardStream::stdout(self.color_choice);
        stream
    }

    pub fn create_writer(&self) -> impl std::io::Write {
        //let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
        let stream = BufferedStandardStream::stdout(self.color_choice);
        stream
    }
    pub fn create_printer(&self) -> Standard<BufferedStandardStream>{
        // let match_spec_color: UserColorSpec = "match:fg:255,197,12".parse().unwrap();
        // let line_spec_color: UserColorSpec = "line:fg:1,246,238".parse().unwrap();
        // let match_spec_underline: UserColorSpec = "match:style:underline".parse().unwrap();
        // let path_spec_color: UserColorSpec = "path:fg:52,154,179".parse().unwrap();
        // let color_specs = ColorSpecs::new(&[match_spec_color, match_spec_underline, line_spec_color, path_spec_color]);
        
        
        let printer = StandardBuilder::new()
            .stats(true)
            .heading(true)
            .per_match(true)
            .only_matching(false)
            .per_match_one_line(true)
            .max_columns(Some(750))
            .max_columns_preview(true)
            .color_specs(self.search_color_specs.to_owned())
            //.build(cli::stdout(ColorChoice::Auto));
            .build(BufferedStandardStream::stdout(ColorChoice::Auto));
        printer
    }

    pub fn create_summary_printer(&self) -> Summary<BufferedStandardStream> {
        let printer = SummaryBuilder::new()
            .stats(true)
            .path(true)
            .build(BufferedStandardStream::stdout(ColorChoice::Auto));

        printer
    }
    pub fn create_searcher() -> Searcher {
        let searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .multi_line(false)
            .line_number(true)
            .build();
        
        searcher
    }
}
impl Default for Settings {
    fn default() -> Self {
        let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
        // let stream = StandardStream::stdout(color_choice);

        Settings {
            
            //searcher: Settings::create_searcher(),
            //overide from toml config
            //color_specs: SearchColorSpecs::default(),
            search_color_specs: ColorSpecs::default_with_color(),
            info_color_spec: ColorSpec::new(),
            use_pdf_tool: AvailablePdfTools::UsePdfDummyTool,
            tools: Box::new(PdfDummyTool {}),
            xpdf_tools_folder: None,
            
            cli: None,
            //override from cli(clap) if defined 
            folder_search_mode: FolderSearchMode::ThreadPerFile,
            print_text: true,
            shorten_line_mode: ShortenLineMode::None,
            
            color_choice,
        }
    }
}
#[derive(Debug)]
pub struct SearchColorSpecs {
    pub match_spec: ColorSpec,
    pub text_spec: ColorSpec,
    pub info_spec: ColorSpec,
    pub extra_spec: ColorSpec,
}


impl Default for SearchColorSpecs {
    fn default() -> Self {
        let mut m_spec = ColorSpec::new();
        m_spec.set_fg(Some(Color::Rgb(255, 197, 12)));
        m_spec.set_bold(true);
        m_spec.set_underline(true);
        m_spec.set_intense(true);

        let mut t_spec = ColorSpec::new();
        t_spec.set_fg(Some(Color::Rgb(249, 246, 238)));

        let mut i_spec = ColorSpec::new();
        i_spec.set_fg(Some(Color::Rgb(52, 154, 179)));

        let mut e_spec = ColorSpec::new();
        e_spec.set_fg(Some(Color::Rgb(181, 235, 18)));

        SearchColorSpecs {
            match_spec: m_spec,
            text_spec: t_spec,
            info_spec: i_spec,
            extra_spec: e_spec,
        }
    }
}

#[ignore]
#[test]
fn test_merge_settings() {
    let settings = Settings::new();
    println!("Settings: {:#?}", settings);
}


