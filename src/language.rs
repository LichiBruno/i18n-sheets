use std::fs;

#[derive(Clone, Debug)]
pub enum Translation {
    Found { key: String, value: String },
    NotFound { key: String },
}

#[derive(Clone, Debug)]
pub struct Language {
    pub code: &'static str,
    pub translations: Vec<Translation>,
}

impl Language {
    pub fn new(code: &'static str) -> Self {
        Self {
            code,
            translations: vec![],
        }
    }

    pub fn generate_keys_ts_file(&self, _path_name: &str, dest_path: &str) {
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

        fs::write(&file_path, &file)
            .expect(&format!("Could not write keys.ts file from {}", self.code));
    }

    pub fn generate_ts_file(&self, _path_name: &str, dest_path: &str) {
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

        fs::write(&file_path, &file).expect(&format!("Could not write ts file for {}", self.code));
    }
}
