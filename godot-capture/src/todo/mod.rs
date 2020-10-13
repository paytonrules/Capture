struct Todo<'a, T: Storage> {
    storage: &'a T,
}

trait Storage {
    fn update(&self, inbox: &String);
}

impl<'a, T> Todo<'a, T>
where
    T: Storage,
{
    fn new(storage: &'a T) -> Self {
        Todo { storage }
    }

    fn save(self, token: String, note: &String) {
        self.storage.update(&format!("- {}", note));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockStorage {
        updated_with: RefCell<String>,
    }

    impl MockStorage {
        fn new() -> Self {
            MockStorage {
                updated_with: RefCell::new("".to_string()),
            }
        }

        fn updated_with(&self, full_update: &String) -> bool {
            self.updated_with.borrow().eq(full_update)
        }
    }

    impl Storage for MockStorage {
        fn update(&self, inbox: &String) {
            *self.updated_with.borrow_mut() = inbox.to_string();
        }
    }

    #[test]
    fn save_note_appends_to_empty_todo_list() {
        let storage = MockStorage::new();
        let todo = Todo::new(&storage);

        todo.save("token".to_string(), &"note".to_string());

        assert!(storage.updated_with(&"- note".to_string()));
    }
}
