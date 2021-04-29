use crate::mongo_id;

#[derive(Clone, Copy, Debug)]
pub struct Pronouns<'a> {
    pub subject: &'a str,
    pub object: &'a str,
    pub possessive_determiner: &'a str,
    pub possessive_pronoun: &'a str,
    pub reflexive: &'a str,
    pub id: mongo_id::MongoId,
    pub popularity: i64,
}

macro_rules! from_doc {
    ($doc:expr) => {
        crate::pronouns::Pronouns {
            subject: $doc.get_str("subject").unwrap_or(""),
            object: $doc.get_str("object").unwrap_or(""),
            possessive_determiner: $doc.get_str("possessive_determiner").unwrap_or(""),
            possessive_pronoun: $doc.get_str("possessive_pronoun").unwrap_or(""),
            reflexive: $doc.get_str("reflexive").unwrap_or(""),
            id: crate::mongo_id::MongoId::new($doc.get_object_id("_id").unwrap().bytes()),
            popularity: $doc.get_i64("popularity").unwrap_or(0),
        };
    };
}
pub(crate) use from_doc;
