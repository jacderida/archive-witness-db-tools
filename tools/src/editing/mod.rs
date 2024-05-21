pub mod fields;
pub mod forms;
pub mod masters;
pub mod news;
pub mod nist_tapes;
#[cfg(test)]
pub mod tests;
pub mod videos;

use db::models::{Person, PersonType};

fn get_people_from_input(
    input: &Vec<String>,
    people: &[Person],
    person_type: PersonType,
) -> Vec<Person> {
    let mut result = Vec::new();
    if !input.is_empty() {
        for name in input {
            let (id, description, historical_title) =
                if let Some(person) = people.iter().find(|p| p.name == *name) {
                    (
                        person.id,
                        person.description.clone(),
                        person.historical_title.clone(),
                    )
                } else {
                    (0, None, None)
                };
            result.push(Person {
                id,
                name: name.to_string(),
                description,
                historical_title,
                types: vec![person_type.clone()],
            });
        }
    }
    result
}
