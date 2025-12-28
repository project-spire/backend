use crate::*;
use crate::generator::*;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct ConstantEntry {
    pub name: Name,
    pub schema: ConstantSchema,
}

#[derive(Debug, Deserialize)]
pub struct ConstantSchema {
    pub name: String,
    pub target: Target,
    #[serde(flatten)] pub scalar: ConstantScalar,
}

#[derive(Debug, Deserialize)]
pub enum ConstantScalar {
    SignedInteger {
        scalar_type: ScalarSignedIntegerType,
        value: i64,
    },
    UnsignedInteger {
        scalar_type: ScalarUnsignedIntegerType,
        value: u64,
    },
    Float {
        scalar_type: ScalarFloatType,
        value: f64,
    },
    String {
        scalar_type: ScalarStringType,
        value: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarSignedIntegerType {
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarUnsignedIntegerType {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarFloatType {
    Float32,
    Float64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScalarStringType {
    String,
}

impl Generator {
    pub fn collect_constant(
        &mut self,
        module: &mut ModuleEntry,
        file: &Path,
        name: Name,
    ) -> Result<(), Error> {
        println!("Collecting constant `{}`", file.display());

        self.register_type(&name.as_type(true))?;

        let schema: ConstantSchema = serde_json::from_str(&fs::read_to_string(file)?)?;

        self.constants.push(ConstantEntry { name, schema });
        module
            .entries
            .push(EntityEntry::ConstantIndex(self.enumerations.len() - 1));

        Ok(())
    }

    pub fn generate_constant(
        &self,
        constant: &ConstantEntry,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        println!("Generating constant `{}`", constant.name.name);

        Ok(())
    }
}
