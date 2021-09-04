use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, Error, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use structopt::StructOpt;

/// PILE: The Rust Geodesy grid management program. Appends grid files into the
/// RG grid stockpile file `assets.pile` and writes metadata files for inclusion
/// into the `assets.zip` directory.
#[derive(StructOpt, Debug)]
#[structopt(name = "pile")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Output file, default pile if not present
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let mut default_dir = dirs::data_local_dir().unwrap_or_default();
    default_dir.push("geodesy");
    eprintln!("Default directory: {}", default_dir.to_str().unwrap());
    let mut pile_path = default_dir.clone();
    if let Some(output) = &opt.output {
        pile_path = PathBuf::from(output);
        default_dir = pile_path.clone();
        default_dir.pop();
        eprintln!("pilepath: {}", pile_path.to_str().unwrap_or_default());
        eprintln!("default_dir: {}", default_dir.to_str().unwrap_or_default());
    } else {
        pile_path.push("assets.pile");
    }
    eprintln!("pilepath: {}", pile_path.to_str().unwrap_or_default());

    // Open `assets.pile` for writing/reading. create if non-existing
    // We cannot use `append` here, since that excludes later partial
    // truncation with `set_len()`, if we have written something, that
    // later shows to be mistakenly written)
    let mut pile = OpenOptions::new()
        .write(true)
        .create(true)
        .open(pile_path)?;
    // Seek to end to make later `stream_position()` calls work correctly
    pile.seek(SeekFrom::End(0))?;

    // Metadata files are placed in the `assets/proj-data` directory below the
    // directory containing the `assets.pile` file. Make sure it exists
    let mut pile_dir = default_dir.clone();
    pile_dir.push("assets");
    pile_dir.push("proj-data");
    std::fs::create_dir_all(&pile_dir)?;

    for path in &opt.files {
        let pos = pile.stream_position()?;
        if "raw" != path.extension().unwrap_or_default() {
            eprintln!(
                "Ignoring non-raw-file arg: {:?}",
                path.file_name().unwrap_or_default()
            );
            continue;
        }

        // Append the source file to the pile
        let mut file = File::open(path)?;
        let mut buffer = Vec::<u8>::new();
        let length = file.read_to_end(&mut buffer)?;
        pile.write(&buffer)?;

        // Check that the writing went well
        let nextpos = pile.stream_position()?;
        assert_eq!((nextpos - pos) as usize, length);

        // Now prepare the aux-file corresponding to the raw-file just written.
        // First we need to locate the original raw file
        let mut aux_path = path.clone();
        aux_path.set_extension("aux");
        let aux_file = File::open(&aux_path);

        // Clean up the pile if no aux-file was found
        if aux_file.is_err() {
            eprintln!(
                "File: {:?} not found - removing the corresponding raw file from pile.",
                aux_path
            );
            pile.set_len(pos)?;
            pile.seek(SeekFrom::End(0))?;
            eprintln!(
                "Was {} bytes. Truncated to {} bytes",
                nextpos,
                pile.stream_position()?
            );
            continue;
        }

        // Line-by-line reading is easier if we Wrap a buffer around the original aux-file
        let aux = std::io::BufReader::new(aux_file?);

        // Now open the corresponding new aux file
        let mut aux_out_path = pile_dir.clone();
        aux_out_path.push(aux_path.file_name().unwrap());
        let mut aux_out = File::create(aux_out_path)?;

        // First line of the new aux file defines the pile-offset of the grid
        let line = format!("Offset: {}\n", pos);
        aux_out.write(line.as_bytes())?;

        // The remaining lines are copied verbatim from the original
        // and supplemented with Geometry and BBox elements
        let mut bbox: HashMap<String, String> = HashMap::new();
        for line in aux.lines() {
            let mut line = line.unwrap().clone();
            let e: Vec<&str> = line.split_whitespace().collect();
            if e[0] == "Offset:" {
                continue;
            }

            // bbox order: s, w, n, e
            if e[0] == "LoRightY:" {
                bbox.insert("s".to_string(), e[1].to_string());
            }
            if e[0] == "UpLeftX:" {
                bbox.insert("w".to_string(), e[1].to_string());
            }
            if e[0] == "UpLeftY:" {
                bbox.insert("n".to_string(), e[1].to_string());
            }
            if e[0] == "LoRightX:" {
                bbox.insert("e".to_string(), e[1].to_string());
            }

            // Make grid geometry easier to read than "RawDefinition: 601 401 1"
            if line.starts_with("RawDefinition:") {
                assert!(e.len() == 4);
                let geometry = format!("Geometry: [{}, {}, {}]\n", e[1], e[2], e[3]);
                aux_out.write(geometry.as_bytes())?;
            }
            line += "\n";
            aux_out.write(line.as_bytes())?;
        }
        assert!(bbox.len() == 4);
        let bboxline = format!(
            "BBox: [{}, {}, {}, {}]\n",
            bbox["s"], bbox["w"], bbox["n"], bbox["e"]
        );
        aux_out.write(bboxline.as_bytes())?;
    }

    if opt.debug {
        eprintln!("{:#?}", opt);
    }

    Ok(())
}
