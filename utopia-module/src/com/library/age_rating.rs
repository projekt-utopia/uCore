#[derive(Debug)]
pub enum PEGIRating {
    Pegi3,
    Pegi7,
    Pegi12,
    Pegi16,
    Pegi18
}

#[derive(Debug)]
pub enum ESRBRating {
    EsrbRatingPending,
    EsrbEarlyChildhood,
    EsrbEveryone,
    EsrbEveryone10Plus,
    EsrbTeen,
    EsrbMature,
    EsrbAdult
}

#[derive(Debug)]
pub enum FSKRating {
    Fsk0,
    Fsk6,
    Fsk12,
    Fsk16,
    Fsk18
}

#[derive(Debug)]
pub struct AgeRating {
    pub pegi_rating: Option<PEGIRating>,
    pub esrb_rating: Option<ESRBRating>,
    pub fsk_rating: Option<FSKRating>
}
