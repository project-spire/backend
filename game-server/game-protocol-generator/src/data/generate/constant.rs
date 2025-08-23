use std::fs;
use crate::data::*;
use crate::data::generate::*;

impl Generator {
    pub fn generate_constant(&self, constant: &ConstantEntry) -> Result<(), GenerateError> {
        let const_dir = self.full_gen_dir(&constant.name.namespaces);
        let code = constant.generate()?;

        fs::write(
            const_dir.join(format!("{}.rs", constant.name.as_entity())),
            code,
        )?;

        Ok(())
    }
}

impl ConstantEntry {
    fn generate(&self) -> Result<String, GenerateError> {
        todo!()
    }
}
