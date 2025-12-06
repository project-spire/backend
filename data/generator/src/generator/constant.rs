use std::fs;
use crate::*;
use crate::generator::*;

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
    pub fn generate_constant(&self, constant: &ConstantEntry) -> Result<(), Error> {
        let constant_dir = self.full_gen_dir(&constant.name.namespace);
        self.log(&format!("Generating constant `{}`", constant_dir.display()));

        let code = constant.generate()?;

        fs::write(
            constant_dir.join(format!("{}.rs", constant.name.as_entity())),
            code,
        )?;

        Ok(())
    }
}

impl ConstantEntry {
    fn generate(&self) -> Result<String, Error> {
        todo!()
    }
}
