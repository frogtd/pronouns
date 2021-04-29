use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
pub struct MongoId {
    pub id: [u8; 12],
}
impl MongoId {
    pub fn new(id: [u8; 12]) -> Self {
        MongoId { id }
    }
}

impl std::fmt::Display for MongoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.id))
    }
}

pub fn str_to_object_id(string_id: &str) -> Result<bson::oid::ObjectId, ()> {
    let object_id_vec = match hex::decode(string_id) {
        Ok(x) => {
            if x.len() == 12 {
                x
            } else {
                return Err(());
            }
        }
        Err(_) => return Err(()),
    };
    let mut object_id = [0; 12];
    for (index, x) in object_id_vec.iter().enumerate() {
        object_id[index] = *x;
    }
    Ok(bson::oid::ObjectId::with_bytes(object_id))
}
