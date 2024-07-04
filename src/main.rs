extern crate walkdir;

use regex::Regex;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::{env, process};
use walkdir::WalkDir;

fn get_absolute_path(name: &str) -> String {
    return [
        env::current_dir().unwrap().display().to_string().as_str(),
        name,
    ]
    .join("\\");
}

// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn show_help() {
    println!(
        r#"
    Description:
    This rust app finds files under a source directory
    using a regex query on the file contents and then
    replaces the matched query with a user provided
    replacement string before copying the files into
    a destination directory.

    Arguments:
        Arg1: -help                 ---- displays this help menu 
        Arg1: Regex Query           ---- The regex query to match within the contents of files
        Arg2: Replacement String    ---- The string to replace the matched query
        Arg3: Source Directory      ---- The directory to search for files
        Arg4: Destination Directory ---- The directory that the files will be copied to

        NOTE: Arg3 and Arg4 starts with the cwd. Arg4 as "..\..\Output" means
              two levels above your current, then into a dir called "Output"

    "#
    );
    process::exit(1);
}

fn get_arg(index: usize, args: &[String]) -> String {
    args.get(index)
        .cloned() // Clone the String to return an owned value
        .expect("Argument Missing (use fcr -help for more information)")
}

fn check_dir(dest: &String) {
    loop {
        let mut response = String::new();
        println!("\nDestination Directory not Found!");
        print!("Create Specified Directory? [Y]es / [N]o / [C]ancel (default: \"Y\"):  ");
        io::stdout().flush().expect("Failed to flush output");
        io::stdin()
            .read_line(&mut response)
            .expect("Response not recognized!");

        match response.trim().to_lowercase().as_str() {
            "n" | "c" => {
                println!("\nExiting Program!");
                process::exit(1);
            }
            "y" | "" => {
                println!("Creating Directory: {}", dest);
                fs::create_dir_all(dest).expect("Failed to create parent directory");
                return;
            }
            _ => {
                println!("Response not recognized!");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!();

    // help is called
    let arg1 = args
        .get(1)
        .expect("Missing Argument 1 (type \"fcr -help\" for more info)");
    if arg1 == "-help" {
        show_help();
    }

    // Get the args
    let query = get_arg(1, &args);
    let repl = get_arg(2, &args);
    let src = get_arg(3, &args);
    let dest = get_arg(4, &args);

    println!("Running FIND-COPY-REPLACE");
    println!(
        "\nFinding query: '{}'\nIn directory: '{}'\nReplacing with: '{}'\nCopying to: {}\n",
        query, src, repl, dest
    );

    let src_path = get_absolute_path(&src);
    let dest_path = get_absolute_path(&dest);
    let regex_query = Regex::new(&query).unwrap();
    let mut matched_files: Vec<PathBuf> = vec![];
    let mut modified_lines: Vec<String> = vec![];

    println!("Checking Files:");
    // grep search through all the files in src_path
    for file in WalkDir::new(src_path)
        .into_iter()
        .filter_map(|file| file.ok())
    {
        // read file; search for query
        if file.metadata().unwrap().is_file() {
            print!("{:>50} \t ... \t ", &file.file_name().to_str().unwrap());
            let mut temp_str = String::new();
            if let Ok(lines) = read_lines(file.path()) {
                for line in lines.map_while(Result::ok) {
                    temp_str.push_str(&line);
                    temp_str.push('\n');
                }
                if regex_query.is_match(&temp_str) {
                    println!(" Match Found! \t ... \t File Processed!");
                    matched_files.push(file.path().to_path_buf());
                    let modi_line = regex_query.replace(&temp_str, &repl);
                    modified_lines.push(modi_line.to_string());
                } else {
                    println!(" No Match Found!");
                }
            }
        }
    }

    // copy files src to dest folder
    for (i, file) in matched_files.iter().enumerate() {
        let content = &modified_lines[i];

        let file_name = file
            .file_name()
            .unwrap()
            .to_str()
            .expect("Invalid File Name!");
        let dest_file_path = [&dest_path, file_name].join("\\");

        // check directory
        if !Path::new(&dest).exists() {
            check_dir(&dest_path);
        }
        println!("\nCopying files to {}:", &dest_path);

        // skip file if already exists at location
        if Path::new(&dest_file_path).exists() {
            // fs::remove_file(&dest_file_path).unwrap();
            eprintln!(
                "COULD NOT COPY FILE: {} already exists in directory!",
                &file_name
            );
            continue;
        }

        // create new file
        OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&dest_file_path)
            .unwrap();

        fs::write(dest_file_path, content).expect("Failed to write to file");
    }
}
