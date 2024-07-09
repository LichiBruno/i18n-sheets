use crate::language::Language;
use calamine::{Reader, Xlsx};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{stdin, stdout, Write};

pub fn read_input(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    let _ = stdout().flush();
    stdin().read_line(&mut input).expect("Failed to read input");
    if let Some('\n') = input.chars().next_back() {
        input.pop();
    }
    if let Some('\r') = input.chars().next_back() {
        input.pop();
    }
    input
}

pub fn write_global_keys_file<R: std::io::Read + std::io::Seek>(
    dest_path: &str,
    keys_files: &[String],
    workbook: &Xlsx<R>,
) {
    let mut file = String::new();
    file.push_str("// File generated automatically. Do not edit manually.\n");

    let mut unique_imports: HashSet<String> = HashSet::new();

    for import_line in keys_files {
        unique_imports.insert(import_line.clone());
    }

    for import_line in unique_imports {
        file.push_str(&import_line);
        file.push('\n');
    }

    file.push_str("\nconst allKeys: { [k: string]: string } = {\n");

    for sheet_name in workbook.sheet_names() {
        let lower_name = sheet_name.to_lowercase();
        file.push_str(&format!("  ...{},\n", lower_name));
    }

    file.push_str("};\n\nexport default allKeys;\n");

    fs::write(&format!("{}/keys.ts", dest_path), &file)
        .expect("Could not write global keys.ts file");
}

pub fn write_language_files<R: std::io::Read + std::io::Seek>(
    dest_path: &str,
    langs: &[Language],
    language_files: &HashMap<&'static str, Vec<String>>,
    workbook: &Xlsx<R>,
) {
    for lang in langs {
        let mut file = String::new();
        file.push_str("// File generated automatically. Do not edit manually.\n");

        for import_line in language_files.get(lang.code).unwrap() {
            file.push_str(import_line);
            file.push('\n');
        }

        file.push_str("\nconst allTranslations: { [k: string]: string } = {\n");

        for sheet_name in workbook.sheet_names() {
            let lower_name = sheet_name.to_lowercase();
            file.push_str(&format!("  ...{},\n", lower_name));
        }

        file.push_str("};\n\nexport default allTranslations;\n");

        fs::write(&format!("{}/{}.ts", dest_path, lang.code), &file).expect(&format!(
            "Could not write combined ts file for {}",
            lang.code
        ));
    }
}
