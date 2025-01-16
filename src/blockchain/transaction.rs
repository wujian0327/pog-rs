pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub signature: String,
    pub paths: Vec<Path>,
}

pub struct Path {
    pub to: String,
    pub signature: String,
}
