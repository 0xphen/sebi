#[derive(Debug, Clone)]
pub struct ImportFact {
    pub module: String,
    pub name: String,
    pub kind: String, // "func"|"memory"|"table"|"global"|"tag"
}

#[derive(Debug, Clone)]
pub struct ExportFact {
    pub name: String,
    pub kind: String,
}
