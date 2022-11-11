use std::{marker::PhantomData, vec, cell::RefCell};

use crate::{
    backend::Backend,
    error::TxError,
    model::{Model, NewModel, VersionType},
    Ref,
};

enum Action<IdType, Data> {
    Create { model: NewModel<IdType, Data> },
    //    Read {
    //        id: IdType,
    //        version: VersionType
    //    },
    Update { model: Model<IdType, Data> },
    Delete { id: IdType, version: VersionType },
    DeleteOption { id: IdType, version: VersionType },
}

const COMMIT_PANIC_MESSAGE: &str = "Cannot commit the transaction";

pub struct Tx<Data, B: Backend<Data>> {
    actions: Vec<Action<B::IdType, Data>>,
    backend: Ref<RefCell<B>>,
    completed: bool,
    phantom_data: PhantomData<Data>,
}

impl<Data, B: Backend<Data>> Tx<Data, B> {
    pub(crate) fn new(backend: Ref<RefCell<B>>) -> Self {
        Self {
            actions: vec![],
            backend,
            completed: false,
            phantom_data: PhantomData,
        }
    }

    pub fn fetch_one(&mut self, id: &B::IdType) -> Result<Model<B::IdType, Data>, TxError> {
        let result = self.backend.borrow().fetch_one(id);
        /*
        match &result {
            Ok(model) => {
                self.actions.push(Action::Read { id: model.id.clone(), version: model.version });
            },
            _ => ()
        };
        */
        result
    }

    pub fn fetch_option_one(
        &mut self,
        id: &B::IdType,
    ) -> Result<Option<Model<B::IdType, Data>>, TxError> {
        let result = self.backend.borrow().fetch_option_one(id);
        /*
        match &result {
            Ok(Some(model)) => {
                self.actions.push(Action::Read { id: model.id.clone(), version: model.version });
            },
            _ => ()
        };
        */
        result
    }

    pub fn update(&mut self, model: Model<B::IdType, Data>) -> Result<(), TxError> {
        self.actions.push(Action::Update { model });
        Ok(())
    }

    pub fn delete(&mut self, id: &B::IdType) -> Result<(), TxError> {
        let result = self.backend.borrow().fetch_version(id);
        match result {
            Ok(version) => {
                self.actions.push(Action::Delete {
                    id: id.clone(),
                    version,
                });
            }
            _ => (),
        };
        Ok(())
    }

    pub fn delete_option(&mut self, id: &B::IdType) -> Result<(), TxError> {
        let result = self.backend.borrow().fetch_option_version(id);
        match result {
            Ok(Some(version)) => {
                self.actions.push(Action::DeleteOption {
                    id: id.clone(),
                    version,
                });
            }
            _ => (),
        };
        Ok(())
    }

    pub fn save(&mut self, model: NewModel<B::IdType, Data>) -> Result<(), TxError> {
        self.actions.push(Action::Create { model });
        Ok(())
    }

    /// Commits the transaction. Panics if any error
    pub fn commit(mut self) {
        self.inner_commit().expect(COMMIT_PANIC_MESSAGE)
    }

    fn inner_commit(&mut self) -> Result<(), TxError> {
        if self.completed {
            return Ok(());
        }

        self.completed = true;

        let mut backend = self.backend.borrow_mut();

        // Step 1: check that models have the expected version
        for action in &self.actions {
            match action {
                Action::Create { model } => {
                    if backend.fetch_option_version(&model.id)?.is_some() {
                        return Err(TxError::SaveError { message: format!("Cannot save model with id [{}] because the id is already in use.", model.id) });
                    }
                },
                // Action::Read { id, version } => {
                //    if let Some(fetch_version) = self.backend.fetch_option_version(id)? {
                //        return Err(TxError::SaveError { message: format!("Cannot save model with id [{}] because the id is already in use.", model.id) });
                //    }
                // }
                Action::Update { model } => {
                    match backend.fetch_option_version(&model.id)? {
                        Some(fetch_version) if fetch_version == model.version => (),
                        Some(fetch_version) => return Err(TxError::UpdateOptimisticLockError { message: format!("Cannot update model with id [{}]. Expected version [{}], version found [{}]", model.id, model.version, fetch_version) }),
                        None => return Err(TxError::UpdateError { message: format!("Cannot update model with id [{}] because it does not exist.", model.id) }),
                    }
                },
                Action::Delete { id, version } => {
                    match backend.fetch_option_version(id)? {
                        Some(fetch_version) if fetch_version == *version => (),
                        Some(fetch_version) => return Err(TxError::DeleteOptimisticLockError { message: format!("Cannot delete model with id [{}]. Expected version [{}], version found [{}]", id, version, fetch_version) }),
                        None => return Err(TxError::DeleteError { message: format!("Cannot delete model with id [{}] because it does not exist.", id) }),
                    }
                },
                Action::DeleteOption { id, version } => {
                    match backend.fetch_option_version(id)? {
                        Some(fetch_version) if fetch_version == *version => (),
                        Some(fetch_version) => return Err(TxError::DeleteOptimisticLockError { message: format!("Cannot delete model with id [{}]. Expected version [{}], version found [{}]", id, version, fetch_version) }),
                        None => (),
                    }
                }
            }
        }

        for action in self.actions.drain(..) {
            match action {
                Action::Create { model } => backend.save(model)?,
                // Action::Read { .. } => (),
                Action::Update { model } => backend.update(model)?,
                Action::Delete { id, version: _ } => backend.delete(&id)?,
                Action::DeleteOption { id, version: _ } => {
                    backend.delete_option(&id).map(|_| ())?
                }
            }
        }

        Ok(())
    }

    pub fn rollback(mut self) {
        self.completed = true;
        // Do nothing
    }
}

#[cfg(test)]
mod test {

    use std::rc::Rc;

    use crate::{backend::hashmap::HashmapBackend, db::IcTx};

    use super::*;

    #[test]
    fn should_commit_a_tx() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = NewModel { id: 1, data: 1123 };

        // Act
        let mut tx = db.tx();
        tx.save(model.clone()).unwrap();
        tx.commit();

        let fetched_model = db.fetch_one(&model.id).unwrap();
        let fetched_model_opt = db.fetch_option_one(&model.id).unwrap();

        // Assert
        assert_eq!(model.id, fetched_model.id);
        assert_eq!(model.data, fetched_model.data);
        assert_eq!(0, fetched_model.version);

        assert_eq!(Some(fetched_model), fetched_model_opt);
    }

    #[test]
    fn should_rollback_a_tx() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = NewModel { id: 1, data: 1123 };

        // Act
        let mut tx = db.tx();
        tx.save(model.clone()).unwrap();
        tx.rollback();

        let fetched_model_opt = db.fetch_option_one(&model.id).unwrap();

        // Assert
        assert!(fetched_model_opt.is_none());
    }

    #[test]
    #[should_panic]
    fn commit_should_panic_if_failure() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));

        // Act
        {
            let mut tx = db.tx();
            tx.update(Model {
                id: 1,
                version: 12,
                data: 1123,
            })
            .unwrap();
            assert!(true, "The update should succeed");

            tx.commit();
            assert!(false, "Should panic before this line");
        }
    }

    #[test]
    fn commit_should_fail_if_concurrent_creation() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model_1 = NewModel { id: 1, data: 1111 };

        let model_2 = NewModel { id: 1, data: 2222 };

        // Act
        let mut tx_1 = db.tx();
        tx_1.save(model_1.clone()).unwrap();

        let tx_2_result = {
            let mut tx_2 = db.tx();
            tx_2.save(model_2.clone()).unwrap();
            tx_2.inner_commit()
        };

        let tx_1_result = tx_1.inner_commit();

        let fetched_model = db.fetch_one(&model_1.id).unwrap();

        // Assert
        assert!(tx_2_result.is_ok());
        assert!(tx_1_result.is_err());

        assert_eq!(model_2.id, fetched_model.id);
        assert_eq!(model_2.data, fetched_model.data);
        assert_eq!(0, fetched_model.version);
    }

    #[test]
    fn commit_should_fail_if_concurrent_update() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model_1 = NewModel { id: 1, data: 1111 };
        {
            let mut tx = db.tx();
            tx.save(model_1.clone()).unwrap();
            tx.commit()
        }
        let model_1 = db.fetch_one(&model_1.id).unwrap();

        // Act
        let mut tx_1 = db.tx();
        tx_1.update(model_1.clone()).unwrap();

        let tx_2_result = {
            let mut tx_2 = db.tx();
            tx_2.update(Model {
                id: model_1.id,
                version: model_1.version,
                data: 2222,
            })
            .unwrap();
            tx_2.inner_commit()
        };

        let tx_1_result = tx_1.inner_commit();

        let fetched_model = db.fetch_one(&model_1.id).unwrap();

        // Assert
        assert!(tx_2_result.is_ok());
        assert!(tx_1_result.is_err());

        assert_eq!(model_1.id, fetched_model.id);
        assert_eq!(2222, fetched_model.data);
        assert_eq!(1, fetched_model.version);
    }

    #[test]
    fn commit_should_fail_if_concurrent_delete() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model_1 = NewModel { id: 1, data: 1111 };
        {
            let mut tx = db.tx();
            tx.save(model_1.clone()).unwrap();
            tx.commit()
        }
        let model_1 = db.fetch_one(&model_1.id).unwrap();

        // Act
        let mut tx_1 = db.tx();
        tx_1.delete(&model_1.id).unwrap();

        let tx_2_result = {
            let mut tx_2 = db.tx();
            tx_2.delete(&model_1.id).unwrap();
            tx_2.inner_commit()
        };

        let tx_1_result = tx_1.inner_commit();

        let fetched_model = db.fetch_option_one(&model_1.id).unwrap();

        // Assert
        assert!(tx_2_result.is_ok());
        assert!(tx_1_result.is_err());

        assert!(fetched_model.is_none());
    }

    #[test]
    fn save_should_save_a_model() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = NewModel { id: 1, data: 1123 };

        // Act
        let mut tx = db.tx();
        tx.save(model.clone()).unwrap();
        tx.commit();

        let fetched_model = db.fetch_one(&model.id).unwrap();
        let fetched_model_opt = db.fetch_option_one(&model.id).unwrap();

        // Assert
        assert_eq!(model.id, fetched_model.id);
        assert_eq!(model.data, fetched_model.data);
        assert_eq!(0, fetched_model.version);

        assert_eq!(Some(fetched_model), fetched_model_opt);
    }

    #[test]
    fn save_should_fail_if_key_exists() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = NewModel { id: 1, data: 1123 };

        // Act
        let first_save = {
            let mut tx = db.tx();
            tx.save(model.clone()).unwrap();
            tx.inner_commit()
        };
        let second_save = {
            let mut tx = db.tx();
            tx.save(model.clone()).unwrap();
            tx.inner_commit()
        };
        let fetched_model = db.fetch_one(&model.id).unwrap();

        // Assert
        assert!(first_save.is_ok());
        assert!(second_save.is_err());
        assert_eq!(model.id, fetched_model.id);
        assert_eq!(model.data, fetched_model.data);
        assert_eq!(0, fetched_model.version);
    }

    #[test]
    fn fetch_one_should_fail_if_missing() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));

        // Act
        let fetched_model_0 = db.fetch_one(&0);
        let fetched_model_1 = db.tx().fetch_one(&0);

        // Assert
        assert!(fetched_model_0.is_err());
        assert!(fetched_model_1.is_err());
    }

    #[test]
    fn fetch_option_one_should_return_none_if_missing() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));

        // Act
        let fetched_model_0 = db.fetch_option_one(&0).unwrap();
        let fetched_model_1 = db.tx().fetch_option_one(&0).unwrap();

        // Assert
        assert!(fetched_model_0.is_none());
        assert!(fetched_model_1.is_none());
    }

    #[test]
    fn update_should_update_a_model() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = NewModel { id: 1, data: 1111 };
        {
            let mut tx = db.tx();
            tx.save(model.clone()).unwrap();
            tx.commit();
        }
        let fetched_model_0 = db.fetch_one(&model.id).unwrap();

        // Act
        let mut tx = db.tx();
        let mut updated_model = fetched_model_0.clone();
        updated_model.data = 2222;
        tx.update(updated_model.clone()).unwrap();
        tx.commit();
        let fetched_model_1 = db.fetch_one(&model.id).unwrap();

        // Assert
        assert_eq!(model.id, fetched_model_0.id);
        assert_eq!(model.data, fetched_model_0.data);
        assert_eq!(0, fetched_model_0.version);

        assert_eq!(model.id, fetched_model_1.id);
        assert_eq!(updated_model.data, fetched_model_1.data);
        assert_eq!(1, fetched_model_1.version);
    }

    #[test]
    fn update_should_fail_if_key_does_not_exists() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = Model {
            id: 1,
            version: 0,
            data: 1111,
        };

        // Act
        let update_result = {
            let mut tx = db.tx();
            tx.update(model.clone()).unwrap();
            tx.inner_commit()
        };
        let fetched_model = db.fetch_option_one(&model.id).unwrap();

        // Assert
        assert!(update_result.is_err());
        assert!(fetched_model.is_none());
    }

    #[test]
    fn update_should_fail_if_version_mismatch() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = NewModel { id: 1, data: 1111 };
        {
            let mut tx = db.tx();
            tx.save(model.clone()).unwrap();
            tx.commit();
        }
        let fetched_model_0 = db.fetch_one(&model.id).unwrap();

        // Act
        let result_1 = {
            let mut tx = db.tx();
            tx.update(fetched_model_0.clone()).unwrap();
            tx.inner_commit()
        };
        // this should fail because the version does not match
        let result_2 = {
            let mut tx = db.tx();
            tx.update(fetched_model_0.clone()).unwrap();
            tx.inner_commit()
        };

        let fetched_model_1 = db.fetch_one(&model.id).unwrap();
        let result_3 = {
            let mut tx = db.tx();
            tx.update(fetched_model_1.clone()).unwrap();
            tx.inner_commit()
        };
        let fetched_model_2 = db.fetch_one(&model.id).unwrap();

        // Assert
        assert_eq!(model.id, fetched_model_0.id);
        assert_eq!(model.data, fetched_model_0.data);
        assert_eq!(0, fetched_model_0.version);

        assert!(result_1.is_ok());
        assert!(result_2.is_err());
        assert!(result_3.is_ok());

        assert_eq!(model.id, fetched_model_1.id);
        assert_eq!(1, fetched_model_1.version);

        assert_eq!(model.id, fetched_model_2.id);
        assert_eq!(2, fetched_model_2.version);
    }

    #[test]
    fn delete_should_delete_a_model() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = NewModel { id: 1, data: 1123 };
        {
            let mut tx = db.tx();
            tx.save(model.clone()).unwrap();
            tx.commit();
        }

        // Act
        let fetched_before = db.fetch_option_one(&model.id).unwrap();
        let delete_result_1 = {
            let mut tx = db.tx();
            tx.delete(&model.id).unwrap();
            tx.inner_commit()
        };
        let fetched_after = db.fetch_option_one(&model.id).unwrap();
        let delete_result_2 = {
            let mut tx = db.tx();
            tx.delete(&model.id).unwrap();
            tx.inner_commit()
        };

        // Assert
        assert!(fetched_before.is_some());
        assert!(delete_result_1.is_ok());
        assert!(fetched_after.is_none());
        assert!(delete_result_2.is_err());
    }

    #[test]
    fn delete_option_should_delete_a_model() {
        // Arrange
        let db = IcTx::new(Rc::new(RefCell::new(HashmapBackend::<i32, i32>::new())));
        let model = NewModel { id: 1, data: 1123 };
        {
            let mut tx = db.tx();
            tx.save(model.clone()).unwrap();
            tx.commit();
        }

        // Act
        let fetched_before = db.fetch_option_one(&model.id).unwrap();
        let delete_result_1 = {
            let mut tx = db.tx();
            tx.delete_option(&model.id).unwrap();
            tx.inner_commit()
        };
        let fetched_after = db.fetch_option_one(&model.id).unwrap();
        let delete_result_2 = {
            let mut tx = db.tx();
            tx.delete_option(&model.id).unwrap();
            tx.inner_commit()
        };

        // Assert
        assert!(fetched_before.is_some());
        assert!(delete_result_1.is_ok());
        assert!(fetched_after.is_none());
        assert!(!delete_result_2.is_ok());
    }
}
