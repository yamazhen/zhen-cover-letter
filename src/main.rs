use clap::{Parser, ValueEnum};
use printpdf::*;
use std::fs;
use std::io::{self, Write};
mod languages;
mod template;

use languages::{english, korean};
use template::CoverLetter;

#[derive(Parser)]
struct Args {
    #[arg(short = 'c')]
    company: Option<String>,

    #[arg(short = 'l', default_value = "en")]
    lang: Language,
}

#[derive(Clone, ValueEnum)]
enum Language {
    En,
    Kr,
}

impl Language {
    fn as_str(&self) -> &str {
        match self {
            Language::En => "en",
            Language::Kr => "kr",
        }
    }
}

fn main() {
    let args = Args::parse();

    let company = args.company.unwrap_or_else(|| prompt("Company name: "));

    let cover_letter = CoverLetter {
        company_name: company.clone(),
    };

    let template = match args.lang {
        Language::En => english::get_template(),
        Language::Kr => korean::get_template(),
    };

    let content = cover_letter.render(template);

    create_pdf(&content, &company, args.lang.as_str());
    println!(
        "Cover letter generated: ./generated/{}/{}.pdf",
        args.lang.as_str(),
        company
    );
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > max_width {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
                current_line.clear();
            }
        }

        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

fn create_pdf(content: &str, company_name: &str, lang: &str) {
    let dir_path = format!("./generated/{}", lang);
    fs::create_dir_all(&dir_path).unwrap();

    let (doc, page1, layer1) = PdfDocument::new("Cover Letter", Mm(210.0), Mm(297.0), "Layer 1");
    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    let font = if lang == "kr" {
        println!("Loading Korean fonts...");
        
        // Use system Korean font only, skip Noto Sans KR as it seems to cause issues
        if let Ok(system_file) = std::fs::File::open("/System/Library/Fonts/AppleSDGothicNeo.ttc") {
            println!("Using system Korean font");
            if let Ok(korean_font) = doc.add_external_font(system_file) {
                korean_font
            } else {
                println!("Failed to add system Korean font, using Helvetica");
                doc.add_builtin_font(BuiltinFont::Helvetica).unwrap()
            }
        } else {
            println!("No Korean fonts found, using Helvetica");
            doc.add_builtin_font(BuiltinFont::Helvetica).unwrap()
        }
    } else {
        doc.add_builtin_font(BuiltinFont::Helvetica).unwrap()
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut y_position = 270.0;
    let margin_bottom = 20.0;
    let mut is_first_header = true;

    for line in lines {
        let trimmed_line = line.trim();
        if !trimmed_line.is_empty() {
            if trimmed_line.starts_with("# ") {
                let header_text = line.trim_start().strip_prefix("# ").unwrap();
                let wrap_limit = if lang == "kr" { 120 } else { 80 };
                let wrapped_header = wrap_text(header_text, wrap_limit);

                if !is_first_header {
                    y_position -= 4.0;
                }

                for wrapped_line in wrapped_header {
                    if y_position < margin_bottom + 12.0 {
                        let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                        current_layer = doc.get_page(new_page).get_layer(new_layer);
                        y_position = 270.0;
                    }
                    // Set blue color for headers
                    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.4, 0.8, None)));
                    current_layer.use_text(
                        &wrapped_line,
                        14.0,
                        Mm(20.0),
                        Mm(y_position),
                        &font,
                    );
                    // Reset to black color for body text
                    current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
                    y_position -= 6.0;
                }
                is_first_header = false;
            } else {
                let wrap_limit = if lang == "kr" { 120 } else { 90 };
                let wrapped_lines = wrap_text(trimmed_line, wrap_limit);

                for wrapped_line in wrapped_lines {
                    if y_position < margin_bottom + 8.0 {
                        let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                        current_layer = doc.get_page(new_page).get_layer(new_layer);
                        y_position = 270.0;
                    }
                    current_layer.use_text(&wrapped_line, 10.0, Mm(20.0), Mm(y_position), &font);
                    y_position -= 6.5;
                }
            }
        } else {
            y_position -= 3.0;
        }
    }

    let file_path = format!("{}/{}.pdf", dir_path, company_name);
    doc.save(&mut std::io::BufWriter::new(
        std::fs::File::create(file_path).unwrap(),
    ))
    .unwrap();
}

fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
