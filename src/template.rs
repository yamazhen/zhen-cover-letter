pub struct CoverLetter {
    pub company_name: String,
}

impl CoverLetter {
    pub fn render(&self, template: &str) -> String {
        template.replace("${companyName}", &self.company_name)
    }
}
