use crate::language::Language;
use calamine::DataType;
use std::collections::HashMap;

pub struct SheetSchema {
    pub name: String,
    pub prefix_column: usize,
    pub name_column: usize,
    pub translation_columns: HashMap<&'static str, usize>,
}

impl SheetSchema {
    pub fn new(name: String, range: &calamine::Range<DataType>, langs: &[Language]) -> Self {
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

    pub fn get_row_key(&self, row: &[DataType]) -> String {
        format!(
            "{}{}",
            self.get_row_column(row, self.prefix_column, "prefix"),
            self.get_row_column(row, self.name_column, "name")
        )
    }

    pub fn get_row_translation(&self, row: &[DataType], lang: &Language) -> Option<String> {
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

    pub fn get_row_column(&self, row: &[DataType], position: usize, field: &str) -> String {
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
