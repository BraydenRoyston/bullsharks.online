/* Internal */

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Athlete {
    pub id: String,
    pub name: String,
    pub team: String,
    pub event: String
} 