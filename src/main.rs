#[cfg(test)]
mod tests;

mod paradox_data;
mod patches;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write as _},
    path::PathBuf,
};

use crate::paradox_data::ParadoxNode;

#[derive(Debug)]
struct PatcherOptions {
    anbennar_path: PathBuf,
    uhw_path: PathBuf,
}

fn parse_args() -> PatcherOptions {
    let mut positional: usize = 0;
    let mut anbennar_path: Option<PathBuf> = None;
    let mut uhw_path: Option<PathBuf> = None;

    for arg in std::env::args().skip(1) {
        if arg.starts_with("-") {
            continue;
        }

        match positional {
            0 => anbennar_path = Some(arg.into()),
            1 => uhw_path = Some(arg.into()),
            _ => panic!("Too many positional arguments!"),
        }

        positional += 1;
    }

    PatcherOptions {
        anbennar_path: match anbennar_path {
            Some(x) if !x.is_dir() => panic!("Anbennar path not a directory!"),
            Some(x) => x,
            None => panic!("Anbennar path not specified!"),
        },
        uhw_path: match uhw_path {
            Some(x) if !x.is_dir() => panic!("UHW path not a directory!"),
            Some(x) => x,
            None => panic!("UHW path not specified!"),
        },
    }
}

fn create_utf8_bom<P: AsRef<std::path::Path>>(path: P) -> Result<File, std::io::Error> {
    let utf8_bom_signature = b"\xEF\xBB\xBF";
    let mut f = std::fs::File::create(path.as_ref())?;
    f.write_all(utf8_bom_signature)?;

    Ok(f)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_args();

    let output_directory = PathBuf::from("output");
    #[expect(unused_must_use)]
    std::fs::create_dir(&output_directory);

    let mut patches: HashMap<PathBuf, Vec<ParadoxNode>> = HashMap::new();

    println!("Processing mines...");
    patches.insert(
        "anb_uhw_03_mines_GENERATED.txt".into(),
        patches::patch_mines(&options.anbennar_path, &options.uhw_path),
    );
    println!("Processing construction...");
    patches.insert(
        "anb_uhw_13_construction_GENERATED.txt".into(),
        patches::patch_construction(&options.anbennar_path),
    );
    println!("Processing industry...");
    patches.insert(
        "anb_uhw_01_industry_GENERATED.txt".into(),
        patches::patch_industry(&options.anbennar_path, &options.uhw_path),
    );
    println!("Processing plantations...");
    patches.insert(
        "anb_uhw_04_plantations_GENERATED.txt".into(),
        patches::patch_plantations(&options.anbennar_path),
    );
    println!("Processing misc resources...");
    patches.insert(
        "anb_uhw_09_misc_resource_GENERATED.txt".into(),
        patches::patch_misc_resource(&options.anbennar_path, &options.uhw_path),
    );

    for (filename, patches) in patches.iter() {
        let f = create_utf8_bom(output_directory.join(filename))?;
        let mut writer = BufWriter::new(f);

        println!("Writing to {}...", filename.display());
        for patch in patches {
            writeln!(writer, "{patch}")?;
        }
    }

    Ok(())
}
