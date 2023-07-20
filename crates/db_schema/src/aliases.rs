use crate::schema::person;

diesel::alias!(person as person1: Person1, person as person2: Person2);
