use async_trait::async_trait;

#[async_trait]
pub trait ReadonlyFilesystem {
    type Error;

    async fn read(&self, path: &std::path::PathBuf) -> Result<String, Self::Error>;
}

#[derive(Default)]
pub struct EmptyFilesystem;

#[async_trait]
impl ReadonlyFilesystem for EmptyFilesystem {
    type Error = std::io::Error;

    async fn read(&self, _path: &std::path::PathBuf) -> Result<String, Self::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ))
    }
}

pub struct PhysicalFilesystem {
    pub entry_point: std::path::PathBuf,
}

#[async_trait]
impl ReadonlyFilesystem for PhysicalFilesystem {
    type Error = std::io::Error;

    async fn read(&self, path: &std::path::PathBuf) -> Result<String, Self::Error> {
        let path = self.entry_point.join(path);
        Ok(std::fs::read_to_string(path)?)
    }
}
