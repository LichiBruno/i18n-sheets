use calamine::{open_workbook, DataType, Range, Reader, Xlsx};
use std::collections::HashMap;
use std::io::{stdin, stdout, Write};
use std::collections::HashSet;

fn main() {
    let mut path = String::new();
    print!("Enter xlsx file path: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut path)
        .expect("That file path is unreachable");
    if let Some('\n') = path.chars().next_back() {
        path.pop();
    }
    if let Some('\r') = path.chars().next_back() {
        path.pop();
    }

    let mut dest_path = String::new();
    print!("Enter destination folder path: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut dest_path)
        .expect("Failed to read destination folder path");
    if let Some('\n') = dest_path.chars().next_back() {
        dest_path.pop();
    }
    if let Some('\r') = dest_path.chars().next_back() {
        dest_path.pop();
    }

    let mut workbook: Xlsx<_> =
        open_workbook(std::path::Path::new(&path)).expect("Cannot open file");

    let mut langs = vec![
        Language::new("en"),
        Language::new("de"),
        Language::new("pt-br"),
        Language::new("es"),
        Language::new("ru"),
    ];

    let mut language_files = HashMap::new();
        let mut keys_files: Vec<String> = vec![];

    for lang in &langs {
        language_files.insert(lang.code, vec![]);
    }

    for (name, range) in workbook.worksheets() {
        if name.to_lowercase() == "archive" {
            continue;
        }
        
        let schema = SheetSchema::new(name.clone(), &range, &langs);

        for row in range.rows().skip(1) {
            if row.iter().all(|v| v.is_empty()) {
                break;
            }
            let key = schema.get_row_key(row);

            for lang in langs.iter_mut() {
                let t = if let Some(value) = schema.get_row_translation(row, lang) {
                    Translation::Found {
                        key: key.clone(),
                        value,
                    }
                } else {
                    Translation::NotFound { key: key.clone() }
                };

                lang.translations.push(t);
            }
        }

        let lower_name = name.to_lowercase();

        std::fs::create_dir_all(&format!("{}/{}", dest_path, lower_name))
            .expect(&format!("Could not create folders for each sheet"));
        for lang in &langs {
            lang.generate_ts_file(&lower_name, &dest_path);
            lang.generate_keys_ts_file(&lower_name, &dest_path);
            keys_files.push(format!("import {0} from \"./{0}/keys\";", lower_name));
            language_files
                .get_mut(lang.code)
                .unwrap()
                .push(format!("import {1} from \"./{1}/{0}\";", lang.code, lower_name));
        }

        for lang in langs.iter_mut() {
            lang.translations.clear();
        }
    }

    let mut file = String::new();
    file.push_str("// File generated automatically. Do not edit manually.\n");
    
    let mut unique_imports: HashSet<String> = HashSet::new();
    
    for import_line in &keys_files {
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
    
    std::fs::write(&format!("{}/keys.ts", dest_path), &file)
        .expect("Could not write global keys.ts file");
    


    for lang in &langs {
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

        std::fs::write(&format!("{}/{}.ts", dest_path, lang.code), &file)
            .expect(&format!("Could not write combined ts file for {}", lang.code));
    }

}

#[derive(Clone, Debug)]
enum Translation {
    Found { key: String, value: String },
    NotFound { key: String },
}

#[derive(Clone, Debug)]
struct Language {
    code: &'static str,
    translations: Vec<Translation>,
}

impl Language {
    fn new(code: &'static str) -> Self {
        Self {
            code,
            translations: vec![],
        }
    }

    fn generate_keys_ts_file(&self, _path_name: &str, dest_path: &str) {
        let mut file = String::new();

        file.push_str("// File generated automatically. Do not edit manually.\n");
        file.push_str("const keys: { [k: string]: string } = {\n");
        for t in &self.translations {
            let line = match t {
                Translation::Found { key, .. } => format!("  {key}: \"{key}\",\n"),
                Translation::NotFound { key } => format!("  {key}: \"{key}\",\n"),
            };
            file.push_str(&line);
        }
        file.push_str("};\n\nexport default keys;\n");

        let file_path = format!("{}/{}/keys.ts", dest_path, _path_name);
    
        std::fs::write(&file_path, &file)
            .expect(&format!("Could not write keys.ts file from {}", self.code));
    }

    fn generate_ts_file(&self, _path_name: &str, dest_path: &str) {
        let mut file = String::new();

        file.push_str("// File generated automatically. Do not edit manually.\n");
        file.push_str("import keys from \"./keys\";\n\n");
        file.push_str("const translate: { [k: string]: string } = {\n");
        for t in &self.translations {
            let line = match t {
                Translation::Found { key, value } => format!("  [keys.{key}]: \"{value}\",\n"),
                Translation::NotFound { key } => {
                    format!("//  [keys.{key}]: \"translation_missing\",\n")
                }
            };
            file.push_str(&line);
        }
        file.push_str("};\n\nexport default translate;\n");

        let file_path = format!("{}/{}/{}.ts", dest_path, _path_name, self.code);
    
        std::fs::write(&file_path, &file)
            .expect(&format!("Could not write ts file for {}", self.code));
    }

}

struct SheetSchema {
    name: String,
    prefix_column: usize,
    name_column: usize,
    translation_columns: HashMap<&'static str, usize>,
}

impl SheetSchema {
    fn new(name: String, range: &Range<DataType>, langs: &[Language]) -> Self {
        let header = range
            .rows()
            .next()
            .expect(&format!("Sheet {} had no header row", name));

        let find_header = |needle| {
            header.iter().position(|v| {
                v.get_string()
                    .map(|l| &l.to_lowercase() == needle)
                    .unwrap_or(false)
            })
        };

        let prefix_column =
            find_header("prefix").expect(&format!("Sheet {} had no Prefix column", name));

        let name_column = find_header("name").expect(&format!("Sheet {} had no Name column", name));

        let mut translation_columns = HashMap::new();

        for lang in langs {
            if let Some(index) = find_header(lang.code) {
                translation_columns.insert(lang.code, index);
            } else {
                println!("Column for {} in sheet {} is missing", lang.code, name);
            }
        }

        Self {
            name,
            prefix_column,
            name_column,
            translation_columns,
        }
    }

    fn get_row_key(&self, row: &[DataType]) -> String {
        format!(
            "{}{}",
            self.get_row_column(row, self.prefix_column, "prefix"),
            self.get_row_column(row, self.name_column, "name")
        )
    }

    fn get_row_translation(&self, row: &[DataType], lang: &Language) -> Option<String> {
        let index = self.translation_columns.get(lang.code)?;
        match row.get(*index) {
            None | Some(DataType::Empty) => None,
            Some(DataType::String(value)) => Some(value.to_string()),
            _ => panic!(
                "Bad translation for lang {} in sheet {} key {}",
                lang.code,
                self.name,
                self.get_row_key(row)
            ),
        }
    }

    fn get_row_column(&self, row: &[DataType], position: usize, field: &str) -> String {
        match row.get(position) {
            None => String::new(),
            Some(cell) => match cell {
                DataType::String(value) => value.to_string(),
                DataType::Empty => String::new(),
                _ => {
                    println!(
                        "Warning: Bad {}, not a string, in sheet {}, {:?}",
                        field, self.name, &row
                    );
                    String::new()
                }
            },
        }
    }
}
 