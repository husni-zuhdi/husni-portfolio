#[derive(Debug, Clone)]
pub struct ProfileData;

#[derive(Debug, Clone)]
pub struct BlogsData {
    pub blogs: Vec<BlogData>,
}

#[derive(Debug, Clone)]
pub struct BlogData {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub body: String,
}
