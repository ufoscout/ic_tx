use std::{marker::PhantomData, vec};

use crate::{backend::Backend, Ref, model::{VersionType, Model, NewModel}, error::TxError};

enum Action<IdType, Data> {
    Create {
        model: NewModel<IdType, Data>
    },
//    Read {
//        id: IdType,
//        version: VersionType
//    },
    Update {
        model: Model<IdType, Data>
    },
    Delete {
        id: IdType,
        version: VersionType
    },
    DeleteOption {
        id: IdType,
        version: VersionType
    },
}

pub struct Tx<Data, B: Backend<Data>> {
    actions: Vec<Action<B::IdType, Data>>,
    backend: Ref<B>,
    phantom_data: PhantomData<Data>,
}

impl <Data, B: Backend<Data>> Tx<Data, B> {

    pub(crate) fn new(backend: Ref<B>) -> Self {
        Self { 
            actions: vec![],
            backend, 
            phantom_data: PhantomData,
        }
    }

    pub fn fetch_one(&mut self, id: &B::IdType) -> Result<Model<B::IdType, Data>, TxError> {
        let result = self.backend.fetch_one(id);
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

    pub fn fetch_option_one(&mut self, id: &B::IdType) -> Result<Option<Model<B::IdType, Data>>, TxError> {
        let result = self.backend.fetch_option_one(id);
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
        let result = self.backend.fetch_version(id);
        match result {
            Ok(version) => {
                self.actions.push(Action::Delete { id: id.clone(), version });
            },
            _ => ()
        };
        Ok(())
    }
    
    pub fn delete_option(&mut self, id: &B::IdType) -> Result<(), TxError> {
        let result = self.backend.fetch_option_version(id);
        match result {
            Ok(Some(version)) => {
                self.actions.push(Action::DeleteOption { id: id.clone(), version });
            },
            _ => ()
        };
        Ok(())
    }

    pub fn save(&mut self, model: NewModel<B::IdType, Data>) -> Result<(), TxError> {
        self.actions.push(Action::Create { model });
        Ok(())
    }

    pub fn commit(self) -> Result<(), TxError> {
        // Step 1: check that models have the expected version
        for action in &self.actions {
            match action {
                Action::Create { model } => {
                    if self.backend.fetch_option_version(&model.id)?.is_some() {
                        return Err(TxError::SaveError { message: format!("Cannot save model with id [{}] because the id is already in use.", model.id) });
                    }
                },
                // Action::Read { id, version } => {
                //    if let Some(fetch_version) = self.backend.fetch_option_version(id)? {
                //        return Err(TxError::SaveError { message: format!("Cannot save model with id [{}] because the id is already in use.", model.id) });
                //    }
                // }
                Action::Update { model } => {
                    match self.backend.fetch_option_version(&model.id)? {
                        Some(fetch_version) if fetch_version == model.version => (),
                        Some(fetch_version) => return Err(TxError::UpdateOptimisticLockError { message: format!("Cannot update model with id [{}]. Expected version [{}], version found [{}]", model.id, model.version, fetch_version) }),
                        None => return Err(TxError::UpdateError { message: format!("Cannot update model with id [{}] because it does not exist.", model.id) }),
                    }
                },
                Action::Delete { id, version } => {
                    match self.backend.fetch_option_version(id)? {
                        Some(fetch_version) if fetch_version == *version => (),
                        Some(fetch_version) => return Err(TxError::DeleteOptimisticLockError { message: format!("Cannot delete model with id [{}]. Expected version [{}], version found [{}]", id, version, fetch_version) }),
                        None => return Err(TxError::DeleteError { message: format!("Cannot delete model with id [{}] because it does not exist.", id) }),
                    }
                },
                Action::DeleteOption { id, version } => {
                    match self.backend.fetch_option_version(id)? {
                        Some(fetch_version) if fetch_version == *version => (),
                        Some(fetch_version) => return Err(TxError::DeleteOptimisticLockError { message: format!("Cannot delete model with id [{}]. Expected version [{}], version found [{}]", id, version, fetch_version) }),
                        None => (),
                    }
                }
            }
        }

        for action in self.actions {
            match action {
                Action::Create { model } => self.backend.save(model)?,
                // Action::Read { .. } => (),
                Action::Update { model } => self.backend.update(model)?,
                Action::Delete { id,  version: _ } => self.backend.delete(&id)?,
                Action::DeleteOption { id, version: _ } => self.backend.delete_option(&id).map(|_| ())?,
            }
        }

        Ok(())
    }

    pub fn rollback(self) -> Result<(), TxError> {
        Ok(())
    }

}


#[cfg(test)]
mod test {

    use std::rc::Rc;

    use crate::{db::TxMx, backend::hashmap::HashmapBackend};

    use super::*;

    #[test]
    fn should_commit_a_tx() {
        // Arrange
        let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
        let model = NewModel {
            id: 1,
            data: 1123,
        };

        // Act
        let mut tx = db.tx();
        tx.save(model.clone()).unwrap();
        tx.commit().unwrap();

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
        let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
        let model = NewModel {
            id: 1,
            data: 1123,
        };

        // Act
        let mut tx = db.tx();
        tx.save(model.clone()).unwrap();
        tx.rollback().unwrap();

        let fetched_model_opt = db.fetch_option_one(&model.id).unwrap();
        
        // Assert
        assert!(fetched_model_opt.is_none());

    }

    // #[test]
    // fn save_should_save_a_model() {
    //     // Arrange
    //     let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
    //     let model = NewModel {
    //         id: 1,
    //         data: 1123,
    //     };

    //     // Act
    //     backend.save(model.clone()).unwrap();
    //     let fetched_model = backend.fetch_one(&model.id).unwrap();
    //     let fetched_model_opt = backend.fetch_option_one(&model.id).unwrap();
        
    //     // Assert
    //     assert_eq!(model.id, fetched_model.id);
    //     assert_eq!(model.data, fetched_model.data);
    //     assert_eq!(0, fetched_model.version);

    //     assert_eq!(Some(fetched_model), fetched_model_opt);

    // }

    // #[test]
    // fn save_should_fail_if_key_exists() {
    //     // Arrange
    //     let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
    //     let model = NewModel {
    //         id: 1,
    //         data: 1123,
    //     };

    //     // Act
    //     let first_save = backend.save(model.clone());
    //     let second_save = backend.save(model.clone());
    //     let fetched_model = backend.fetch_one(&model.id).unwrap();
        
    //     // Assert
    //     assert!(first_save.is_ok());
    //     assert!(second_save.is_err());
    //     assert_eq!(model.id, fetched_model.id);
    //     assert_eq!(model.data, fetched_model.data);
    //     assert_eq!(0, fetched_model.version);

    // }

    // #[test]
    // fn fetch_one_should_fail_if_missing() {
    //     // Arrange
    //     let backend = HashmapBackend::<i32, i32>::new();

    //     // Act
    //     let fetched_model = backend.fetch_one(&0);
        
    //     // Assert
    //     assert!(fetched_model.is_err());

    // }

    // #[test]
    // fn fetch_option_one_should_return_none_if_missing() {
    //     // Arrange
    //     let backend = HashmapBackend::<i32, i32>::new();

    //     // Act
    //     let fetched_model = backend.fetch_option_one(&0).unwrap();
        
    //     // Assert
    //     assert!(fetched_model.is_none());

    // }

    // #[test]
    // fn should_return_the_version() {
    //     // Arrange
    //     let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
    //     let model = NewModel {
    //         id: 1,
    //         data: 1123,
    //     };

    //     // Act
    //     backend.save(model.clone()).unwrap();
    //     let fetched_version = backend.fetch_version(&model.id).unwrap();
        
    //     // Assert
    //     assert_eq!(0, fetched_version);

    // }

    // #[test]
    // fn fetch_version_should_fail_if_missing() {
    //     // Arrange
    //     let backend = HashmapBackend::<i32, i32>::new();

    //     // Act
    //     let fetched_model = backend.fetch_version(&0);
        
    //     // Assert
    //     assert!(fetched_model.is_err());

    // }

    // #[test]
    // fn fetch_option_version_should_return_none_if_missing() {
    //     // Arrange
    //     let backend = HashmapBackend::<i32, i32>::new();

    //     // Act
    //     let fetched_model = backend.fetch_option_version(&0).unwrap();
        
    //     // Assert
    //     assert!(fetched_model.is_none());

    // }

    // #[test]
    // fn update_should_update_a_model() {
    //     // Arrange
    //     let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
    //     let model = NewModel {
    //         id: 1,
    //         data: 1111,
    //     };
    //     backend.save(model.clone()).unwrap();
    //     let fetched_model_0 = backend.fetch_one(&model.id).unwrap();

    //     // Act
    //     let mut updated_model = fetched_model_0.clone();
    //     updated_model.data = 2222;
    //     backend.update(updated_model.clone()).unwrap();
    //     let fetched_model_1 = backend.fetch_one(&model.id).unwrap();
        
    //     // Assert
    //     assert_eq!(model.id, fetched_model_0.id);
    //     assert_eq!(model.data, fetched_model_0.data);
    //     assert_eq!(0, fetched_model_0.version);

    //     assert_eq!(model.id, fetched_model_1.id);
    //     assert_eq!(updated_model.data, fetched_model_1.data);
    //     assert_eq!(1, fetched_model_1.version);
    // }

    // #[test]
    // fn update_should_fail_if_key_does_not_exists() {
    //     // Arrange
    //     let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
    //     let model = Model {
    //         id: 1,
    //         version: 0,
    //         data: 1111,
    //     };

    //     // Act
    //     let update_result = backend.update(model.clone());
    //     let fetched_model = backend.fetch_option_one(&model.id).unwrap();
        
    //     // Assert
    //     assert!(update_result.is_err());
    //     assert!(fetched_model.is_none());

    // }

    // #[test]
    // fn update_should_fail_if_version_mismatch() {
    //     // Arrange
    //     let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
    //     let model = NewModel {
    //         id: 1,
    //         data: 1111,
    //     };
    //     backend.save(model.clone()).unwrap();
    //     let fetched_model_0 = backend.fetch_one(&model.id).unwrap();

    //     // Act
    //     let result_1 = backend.update(fetched_model_0.clone());
    //     // this should fail because the version does not match
    //     let result_2 = backend.update(fetched_model_0.clone());
        
    //     let fetched_model_1 = backend.fetch_one(&model.id).unwrap();
    //     let result_3 = backend.update(fetched_model_1.clone());
    //     let fetched_model_2 = backend.fetch_one(&model.id).unwrap();
        
    //     // Assert
    //     assert_eq!(model.id, fetched_model_0.id);
    //     assert_eq!(model.data, fetched_model_0.data);
    //     assert_eq!(0, fetched_model_0.version);

    //     assert!(result_1.is_ok());
    //     assert!(result_2.is_err());
    //     assert!(result_3.is_ok());

    //     assert_eq!(model.id, fetched_model_1.id);
    //     assert_eq!(1, fetched_model_1.version);

    //     assert_eq!(model.id, fetched_model_2.id);
    //     assert_eq!(2, fetched_model_2.version);
    // }

    // #[test]
    // fn delete_should_delete_a_model() {
    //     // Arrange
    //     let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
    //     let model = NewModel {
    //         id: 1,
    //         data: 1123,
    //     };
    //     backend.save(model.clone()).unwrap();

    //     // Act
    //     let fetched_before = backend.fetch_option_one(&model.id).unwrap();
    //     let delete_result_1 = backend.delete(&model.id);
    //     let fetched_after = backend.fetch_option_one(&model.id).unwrap();
    //     let delete_result_2 = backend.delete(&model.id);
        
    //     // Assert
    //     assert!(fetched_before.is_some());
    //     assert!(delete_result_1.is_ok());
    //     assert!(fetched_after.is_none());
    //     assert!(delete_result_2.is_err());

    // }

    // #[test]
    // fn delete_option_should_delete_a_model() {
    //     // Arrange
    //     let db = TxMx::new(Rc::new(HashmapBackend::<i32, i32>::new()));
    //     let model = NewModel {
    //         id: 1,
    //         data: 1123,
    //     };
    //     backend.save(model.clone()).unwrap();

    //     // Act
    //     let fetched_before = backend.fetch_option_one(&model.id).unwrap();
    //     let delete_result_1 = backend.delete_option(&model.id).unwrap();
    //     let fetched_after = backend.fetch_option_one(&model.id).unwrap();
    //     let delete_result_2 = backend.delete_option(&model.id).unwrap();
        
    //     // Assert
    //     assert!(fetched_before.is_some());
    //     assert!(delete_result_1);
    //     assert!(fetched_after.is_none());
    //     assert!(!delete_result_2);

    // }

}