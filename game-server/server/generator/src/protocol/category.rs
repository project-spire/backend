use serde::Deserialize;
use std::fs;
use crate::protocol::Config;

const TAB: &str = "    ";

impl Config {
    pub fn generate_category(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("cargo:rerun-if-changed={}", self.category_path.display());

        let categories: Vec<Category> = serde_json::from_str(
            &fs::read_to_string(&self.category_path)?
        )?;

        let mut category_definitions = Vec::new();
        let mut category_matches = Vec::new();

        for category in categories {
            category_definitions.push(format!(
                "{TAB}{} = {},",
                category.name,
                category.value,
            ));

            category_matches.push(format!(
                "{TAB}{TAB}{TAB}{} => Self::{},",
                category.value,
                category.name,
            ));
        }

        let code = format!(
r#"//Generated code
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum Category {{
{category_definitions_code}
}}

impl Category {{
    pub fn decode(value: u8) -> Result<Self, crate::protocol::Error> {{
        Ok(match value {{
{category_matches_code}
            _ => return Err(crate::protocol::Error::InvalidCategory(value)),
        }})
    }}
}}
"#,
            category_definitions_code = category_definitions.join("\n"),
            category_matches_code = category_matches.join("\n"),
        );

        fs::write(&self.category_gen_path, &code)?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct Category {
    name: String,
    value: u8,
}
