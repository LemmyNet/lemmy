use crate::schema::person;

diesel::alias! {
    const PERSON_1 = person as person_1: Person1, person as person_2: Person2
};
