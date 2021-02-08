use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum PEGIRating {
    Pegi3,
    Pegi7,
    Pegi12,
    Pegi16,
    Pegi18
}

#[derive(Debug, Serialize)]
pub enum ESRBRating {
    EsrbRatingPending,
    EsrbEarlyChildhood,
    EsrbEveryone,
    EsrbEveryone10Plus,
    EsrbTeen,
    EsrbMature,
    EsrbAdult
}

#[derive(Debug, Serialize)]
pub enum FSKRating {
    Fsk0,
    Fsk6,
    Fsk12,
    Fsk16,
    Fsk18
}

#[derive(Debug, Serialize)]
pub struct AgeRating {
    pegi_rating: Option<PEGIRating>,
    esrb_rating: Option<ESRBRating>,
    fsk_rating: Option<FSKRating>
}
