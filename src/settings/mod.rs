
use std::io::IsTerminal;
use std::path::PathBuf;

use grep::searcher::{Searcher, SearcherBuilder};
use termcolor::{ColorChoice, WriteColor};
use termcolor::{Color, StandardStream, ColorSpec};

use clap::{Parser, Subcommand};
use crate::pdf_tools::{PDFTools, PdfTestTools};

use self::toml_settings::{ConfigColorSpec, TomlSettings};

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
#[derive(PartialEq, Debug)]
pub enum FolderSearchMode {
    ThreadPerFolder,
    ThreadPerFile,
}

#[derive(Clone, Copy, Debug)]
pub enum ShortenLineMode {
    None,
    Trim(usize),
}
#[derive(Debug)]
pub struct Settings {
    // pub stream: StandardStream,
    // pub searcher: Searcher,
    
    pub color_specs: SearchColorSpecs,
    pub tools: Box<dyn PDFTools 
        + std::marker::Send // needed for threads
	    + std::marker::Sync
        >,
    pub folder_search_mode: FolderSearchMode,
    pub print_text: bool,
    pub shorten_line_mode: ShortenLineMode,
    pub color_choice: ColorChoice,

    pub cli: Option<Cli>,
    pub xpdf_tools_folder: Option<PathBuf>,
    
}

impl Settings {
    pub fn new() -> Self {
        let mut settings = Settings::default();

        Settings::merge_toml_settings(&mut settings);
        settings.cli = Some(Cli::parse());

        settings
    }

    fn merge_toml_settings(settings: &mut Settings) {
        let toml = TomlSettings::load()
            .unwrap_or_else(|_err| TomlSettings::create_default()
                .unwrap_or( TomlSettings { xpdf_tools_folder: None, colors: vec![]}));
        
        settings.xpdf_tools_folder = toml.xpdf_tools_folder;
        
        for color_item in toml.colors {
            match color_item.name.as_str() {
                "match" => { settings.color_specs.match_spec = Settings::create_color_item(&color_item); },
                "text" => { settings.color_specs.text_spec = Settings::create_color_item(&color_item); },
                "info" => { settings.color_specs.info_spec = Settings::create_color_item(&color_item); },
                "extra" => { settings.color_specs.extra_spec = Settings::create_color_item(&color_item); },
                _ => (),
            }
        }
    }
    fn create_color_item(item: &ConfigColorSpec) -> ColorSpec {
        // let mut m_spec = ColorSpec::new();
        // m_spec.set_fg(Some(Color::Rgb(255, 197, 12)));
        // m_spec.set_bold(true);
        // m_spec.set_underline(true);
        // m_spec.set_intense(true);
        // ("bold".into(), false), ("intense".into(), false), ("underline".into(), false),
        //         ("dimmed".into(), false), ("italic".into(), false), ("reset".into(), false), ("strikethrough".into(), false)

        let mut spec = ColorSpec::new();
        if let Some(fg) = item.fg {
            spec.set_fg(Some(Color::Rgb(fg.0, fg.1, fg.2)));
        }
        if let Some(bg) = item.bg {
            spec.set_bg(Some(Color::Rgb(bg.0, bg.1, bg.2)));
        }

        for (style, flag) in &item.styles {
            if *flag == true {
                match style.as_str() {
                    "bold" => {spec.set_bold(true);},
                    "intense" => {spec.set_intense(true);},
                    "underline" => {spec.set_underline(true);},
                    "dimmed" => {spec.set_dimmed(true);},
                    "italic" => {spec.set_italic(true);},
                    "reset" => {spec.set_reset(true);},
                    "strikethrough" => {spec.set_strikethrough(true);},
                    _ => (),
                }
            }
        }

        spec
    }
    pub fn create_color_writer(&self) -> impl std::io::Write + WriteColor{
        //let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
        let stream = StandardStream::stdout(self.color_choice);
        stream
    }

    pub fn create_writer(&self) -> impl std::io::Write {
        //let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
        let stream = StandardStream::stdout(self.color_choice);
        stream
    }

    pub fn create_searcher(&self) -> Searcher {
        let searcher = SearcherBuilder::new()
            //.binary_detection(BinaryDetection::quit(b'\x00'))
            //.multi_line(false)
            //.line_number(true)
            .build();
        
        searcher
    }
}
impl Default for Settings {
    fn default() -> Self {
        let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
        // let stream = StandardStream::stdout(color_choice);

        

        Settings {
            //stream,
            //searcher,
            
            color_specs: SearchColorSpecs::default(),
            tools: Box::new(PdfTestTools {}),
            folder_search_mode: FolderSearchMode::ThreadPerFile,
            print_text: true,
            shorten_line_mode: ShortenLineMode::None,
            color_choice,
            cli: None,
            xpdf_tools_folder: None,
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

#[test]
fn test_merge_settings() {
    let settings = Settings::new();
    println!("Settings: {:#?}", settings);
}


