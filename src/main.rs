mod rich_html;

use exitfailure::ExitFailure;
use failure::ResultExt;
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
) -> Result<Vec<std::path::PathBuf>, ExitFailure> {
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

fn extract_ts_number_from_file_path(path: &std::path::PathBuf) -> Option<String> {
    let filename = path.file_stem().unwrap().to_str().unwrap();
    let ts_number_and_version = filename.split("-").collect::<Vec<&str>>();
    if ts_number_and_version.len() < 2 {
        return None;
    }

    let ts_number = ts_number_and_version[0];
    return Some(format!("{}.{}", &ts_number[..2], &ts_number[2..]));
}

fn docx_to_html(
    path: &std::path::PathBuf,
    out_path: &std::path::PathBuf,
) -> Result<String, ExitFailure> {
    let out_path = out_path.to_str().unwrap();
    let output_file_name = format!(
        "{}/{}.html",
        out_path,
        path.file_stem().unwrap().to_string_lossy()
    );

    let _output = std::process::Command::new("lowriter")
        .args(&[
            "--convert-to",
            "html",
            path.to_str().unwrap(),
            &output_file_name,
            "--outdir",
            out_path,
        ])
        .output()
        .with_context(|_| {
            format!(
                "could not convert file `{}` to HTML",
                path.to_str().unwrap()
            )
        })?;

    let html_content = std::fs::read_to_string(&output_file_name)
        .with_context(|_| format!("could not read converted html file `{}`", output_file_name))?;

    Ok(html_content)
}

fn handle_file(
    path: &std::path::PathBuf,
    out_path: &std::path::PathBuf,
) -> Result<(), ExitFailure> {
    let ts_number = extract_ts_number_from_file_path(path);
    if ts_number.is_none() {
        return Err(failure::err_msg(
            "could not extract TS number from file path",
        ))
        .context(format!("file `{}`", path.to_str().unwrap()))?;
    }

    let ts_no = ts_number.unwrap();

    let output_dir = format!("{}/{}", &out_path.to_string_lossy(), ts_no);

    let html_content = docx_to_html(path, &std::path::PathBuf::from(&output_dir))?;
    let html_content = rich_html::enrich_html(&html_content);

    let output_file_path = format!("{}/{}.html", output_dir, ts_no);
    std::fs::write(&output_file_path, &html_content)
        .with_context(|_| format!("could not write HTML file `{}`", &output_file_path))?;

    Ok(())
}

fn main() -> Result<(), ExitFailure> {
    let args = CliArgs::from_args();
    let file_paths = gather_paths_with_extension(&args.input_dir, "doc")?;
    for p in file_paths {
        println!("{}", p.to_string_lossy());
        handle_file(&p, &args.output_dir)?;
    }

    Ok(())
}
