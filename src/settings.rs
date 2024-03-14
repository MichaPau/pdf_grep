use grep::searcher::BinaryDetection;
use grep::searcher::Searcher;
use grep::searcher::SearcherBuilder;
use termcolor::ColorChoice;
use xpdf_tools::XpdfTools;

use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::Path;
use termcolor::{Color, StandardStream, ColorSpec};

use super::BoxError;
pub struct Settings {
    pub stream: StandardStream,
    pub color_specs: SearchColorSpecs,
    pub searcher: Searcher,
    pub tools: Box<dyn PDFTools 
        + std::marker::Send // needed for threads
	    + std::marker::Sync>,
    
}
impl Default for Settings {
    fn default() -> Self {
        let color_choice = if std::io::stdin().is_terminal() { ColorChoice::Auto} else { ColorChoice::Never};
        let stream = StandardStream::stdout(color_choice);

        let searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .multi_line(false)
            .line_number(true)
            .build();

        Settings {
            stream,
            color_specs: SearchColorSpecs::default(),
            searcher,
            tools: Box::new(PdfTestTools {}),
        }
        
    }
}
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

pub trait PDFTools {

    fn pdf_info(&self, file_path: &Path) -> Result<BTreeMap<String, Option<String>>, BoxError>;
    fn pdf_text(&self, file_path: &Path) -> Result<Vec<u8>, BoxError>;
}

pub struct XpdfWrapper {
    pub tools: XpdfTools,
}
pub struct  PdfTestTools {
    
}

impl PDFTools for XpdfWrapper {
    
    fn pdf_info(&self, file_path: &Path) -> Result<BTreeMap<String, Option<String>>, BoxError> {
        match self.tools.pdf_info(file_path) {
            Ok(pdf_info) => Ok(pdf_info.info_map.0),
            Err(e) => Err(Box::new(e)) 
        }
    }

    fn pdf_text(&self, file_path: &Path) -> Result<Vec<u8>, BoxError> {
        match self.tools.pdf_text(file_path) {
            Ok(text) => Ok(text),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl PDFTools for PdfTestTools {
    fn pdf_info(&self, _file_path: &Path) -> Result<BTreeMap<String, Option<String>>, BoxError> {
        let info = BTreeMap::from([
            ("Title".into(), Some("Test tool title".into())), ("Author".into(), Option::None)]);
        Ok(info)
    }

    fn pdf_text(&self, _file_path: &Path) -> Result<Vec<u8>, BoxError> {
        let text = "Some sample text from a test wrapper. \n Can you find something as a test?".as_bytes().to_vec();
        Ok(text)
    }
}
