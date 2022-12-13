use std::collections::HashMap;
use calamine::{Reader, open_workbook, Range, Xlsx, DataType};

fn main() {
  let path = if let Some(filename) = std::env::args().nth(1) {
    filename
  } else {
    println!("Pass in an xlsx file as first argument");
    std::process::exit(1);
  };

  let mut workbook: Xlsx<_> = open_workbook(std::path::Path::new(&path))
    .expect("Cannot open file");

  let mut langs = vec![
    Language::new("en"), 
    Language::new("de"),
    Language::new("pt-br"),
    Language::new("es"),
    Language::new("ru"),
  ];

  for (name, range) in workbook.worksheets() {
    let schema = SheetSchema::new(name, &range, &langs);

    for row in range.rows().skip(1) {
      if row.iter().all(|v| v.is_empty()) {
        break;
      }
      let key = schema.get_row_key(row);

      for lang in langs.iter_mut() {
        let t = if let Some(value) = schema.get_row_translation(row, lang) {
          Translation::Found{ key: key.clone(), value }
        } else {
          Translation::NotFound{ key: key.clone() }
        };

        lang.translations.push(t);
      }
    }
  }

  langs[0].generate_keys_ts_file();

  for lang in &langs {
    lang.generate_ts_file();
  }
}

#[derive(Clone, Debug)]
enum Translation {
  Found{ key: String, value: String},
  NotFound{ key: String },
}

#[derive(Clone, Debug)]
struct Language {
  code: &'static str,
  translations: Vec<Translation>
}

impl Language {
  fn new(code: &'static str) -> Self {
    Self { code, translations: vec![] }
  }

  fn generate_keys_ts_file(&self) {
    let mut file = String::new();
    file.push_str("// File generated automatically. Do not edit manually.\n");
    file.push_str("const keys: { [k: string]: string } = {\n");
    for t in &self.translations {
      let line = match t {
        Translation::Found{ key, .. } => format!("  {key}: \"{key}\",\n"),
        Translation::NotFound{ key } => format!("  {key}: \"{key}\",\n"),
      };
      file.push_str(&line);
    }
    file.push_str("};\n\nexport default keys;\n");
    std::fs::write("build/keys.ts", &file)
      .expect(&format!("Could not write keys.ts file from {}", self.code));
  }

  fn generate_ts_file(&self) {
    let mut file = String::new();

    file.push_str("// File generated automatically. Do not edit manually.\n");
    file.push_str("import keys from \"./keys\";\n\n");
    file.push_str("const translate: { [k: string]: string } = {\n");
    for t in &self.translations {
      let line = match t {
        Translation::Found{ key, value } => format!("  [keys.{key}]: \"{value}\",\n"),
        Translation::NotFound{ key } => format!("//  [keys.{key}]: \"translation_missing\",\n"),
      };
      file.push_str(&line);
    }
    file.push_str("};\n\nexport default translate;\n");

    std::fs::write(&format!("build/{}.ts", self.code), &file)
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
    let header = range.rows().next()
      .expect(&format!("Sheet {} had no header row", name));

    let find_header = |needle|{
      header.iter().position(|v|{
        v.get_string().map(|l| &l.to_lowercase() == needle).unwrap_or(false)
      })
    };

    let prefix_column = find_header("prefix")
      .expect(&format!("Sheet {} had no Prefix column", name));

    let name_column = find_header("name")
      .expect(&format!("Sheet {} had no Name column", name));

    let mut translation_columns = HashMap::new();

    for lang in langs {
      if let Some(index) = find_header(lang.code) {
        translation_columns.insert(lang.code, index);
      } else {
        println!("Column for {} in sheet {} is missing", lang.code, name);
      }
    }

    Self{ name, prefix_column, name_column, translation_columns }
  }

  fn get_row_key(&self, row: &[DataType]) -> String {
    format!("{}{}",
      self.get_row_column(row, self.prefix_column, "prefix"),
      self.get_row_column(row, self.name_column, "name")
    )
  }

  fn get_row_translation(&self, row: &[DataType], lang: &Language) -> Option<String> {
    let index = self.translation_columns.get(lang.code)?;
    match row.get(*index) {
      None | Some(DataType::Empty) => None,
      Some(DataType::String(value)) => Some(value.to_string()),
      _ => panic!("Bad translation for lang {} in sheet {} key {}",
        lang.code, self.name, self.get_row_key(row)),
    }
  }

  fn get_row_column(&self, row: &[DataType], position: usize, field: &str) -> String {
    row.get(position)
      .expect(&format!("No {} column in a row of sheet {}", field, self.name))
      .get_string()
      .expect(&format!("Bad {}, not a string, in sheet {}, {:?}", field, self.name, &row))
      .to_string()
  }
}
