// mod clause_reference_finder;
mod rich_html;

use structopt::StructOpt;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(parse(from_os_str))]
    input_dir: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    output_dir: std::path::PathBuf,
}

fn gather_paths_with_extension(
    dir: &std::path::PathBuf,
    ext: &str,
) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    let files = std::fs::read_dir(&dir)?;

    let mut pdf_paths = Vec::new();

    for f in files {
        let path = &f.unwrap().path();

        if !path.is_file() {
            continue;
        }

        if let Some(path_ext) = path.extension() {
            if path_ext == ext {
                pdf_paths.push(path.clone());
            }
        };
    }

    return Ok(pdf_paths);
}

fn extract_ts_number_from_file_path(path: &std::path::PathBuf) -> Option<&str> {
    // Filename looks like: TSnumber-version.ext
    let filename = path.file_stem().unwrap().to_str().unwrap();

    let ts_number_and_version = filename.split("-").collect::<Vec<&str>>();
    if ts_number_and_version.len() < 2 {
        return None;
    }

    return Some(&ts_number_and_version[0]);
}

fn docx_to_html(path: &std::path::PathBuf) -> Option<String> {
    let output_file_name = format!("tmp/{}.html", path.file_stem().unwrap().to_string_lossy());

    let output = std::process::Command::new("lowriter")
        .args(&[
            "--convert-to",
            "html",
            path.to_str().unwrap(),
            &output_file_name,
            "--outdir",
            "tmp",
        ])
        .output();

    match output {
        Err(_) => {
            return None;
        }
        _ => {}
    }

    let result = std::fs::read_to_string(&output_file_name);
    match result {
        Ok(content) => Some(content),
        Err(_) => None,
    }
}

fn handle_file(path: &std::path::PathBuf) {
    let text_content = docx_to_html(path);
    if text_content.is_none() {
        return;
    }

    let ts_number = &extract_ts_number_from_file_path(path);
    if ts_number.is_none() {
        return;
    }

    let html_content = rich_html::html_to_better_html(&text_content.unwrap());

    std::fs::write("output.html", &html_content);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::from_args();
    println!(
        "Hello, world! in: {}, out: {}",
        args.input_dir.to_string_lossy(),
        args.output_dir.to_string_lossy()
    );

    let pdf_paths = gather_paths_with_extension(&args.input_dir, "docx")?;
    for p in pdf_paths {
        println!("{}", p.to_string_lossy());
        handle_file(&p);
    }

    Ok(())
}
