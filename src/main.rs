mod language;
mod sheet_schema;
mod utils;

use calamine::{open_workbook, Reader, Xlsx};
use language::{Language, Translation};
use sheet_schema::SheetSchema;
use std::collections::HashMap;
use utils::{read_input, write_global_keys_file, write_language_files};

fn main() {
    let path = read_input("Enter xlsx file path: ");
    let dest_path = read_input("Enter destination folder path: ");

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
            .expect("Could not create folders for each sheet");
        for lang in &langs {
            lang.generate_ts_file(&lower_name, &dest_path);
            lang.generate_keys_ts_file(&lower_name, &dest_path);
            keys_files.push(format!("import {0} from \"./{0}/keys\";", lower_name));
            language_files.get_mut(lang.code).unwrap().push(format!(
                "import {1} from \"./{1}/{0}\";",
                lang.code, lower_name
            ));
        }

        for lang in langs.iter_mut() {
            lang.translations.clear();
        }
    }

    write_global_keys_file(&dest_path, &keys_files, &workbook);
    write_language_files(&dest_path, &langs, &language_files, &workbook);
}
