use std::{env, process};

use neoultront::{NeoFontProject, PROJECT_EXTENSION, VERSION};

fn print_help() {
    println!("NeoUltront {VERSION}");
    println!("A font editor project tool");
    println!();
    println!("Usage:");
    println!("  neoultront new <file.{PROJECT_EXTENSION}> [family-name]");
    println!("  neoultront inspect <file.{PROJECT_EXTENSION}>");
    println!("  neoultront validate <file.{PROJECT_EXTENSION}>");
    println!("  neoultront help");
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return Ok(());
    };

    match command.as_str() {
        "new" => {
            let path = args.next().ok_or("missing output .nufont path")?;
            let family = args.next().unwrap_or_else(|| "Untitled Font".to_owned());
            let project = NeoFontProject::new(family);
            project.save(&path)?;
            println!("Created {path}");
        }
        "inspect" => {
            let path = args.next().ok_or("missing .nufont file path")?;
            let project = NeoFontProject::load(&path)?;
            println!("Family: {}", project.metadata.family_name);
            println!("Style: {}", project.metadata.style_name);
            println!("Units per em: {}", project.metadata.units_per_em);
            println!("Glyphs: {}", project.glyphs.len());
            println!("Kerning pairs: {}", project.kerning.len());
            println!("Ligatures: {}", project.ligatures.len());
            println!("Format version: {}", project.format_version);
        }
        "validate" => {
            let path = args.next().ok_or("missing .nufont file path")?;
            NeoFontProject::load(&path)?;
            println!("Valid .{PROJECT_EXTENSION} project: {path}");
        }
        "help" | "--help" | "-h" => print_help(),
        other => {
            print_help();
            return Err(format!("unknown command: {other}").into());
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}
