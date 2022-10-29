use std::{collections::{HashMap, hash_map::Entry}, cell::RefCell, hash::Hash, fmt::Display};

use crate::{model::{Model, NewModel, VersionType}, error::TxError};

use super::Backend;


pub struct HashmapBackend<IdType: Eq + Hash + Clone, Data: Clone> {
    map: RefCell<HashMap<IdType, Model<IdType, Data>>>
}

impl <IdType: Eq + Hash + Clone, Data: Clone> HashmapBackend<IdType, Data> {

    pub fn new() -> Self {
        HashmapBackend { map: RefCell::new(HashMap::default()) }
    }

}

impl <IdType: Eq + Hash + Clone + Display, Data: Clone> Backend<Data> for HashmapBackend<IdType, Data> {

    type IdType = IdType;

    fn fetch_one(&self, id: &Self::IdType) -> Result<Model<Self::IdType, Data>, TxError> {
        match self.fetch_option_one(id) {
            Ok(opt) => opt.ok_or_else(|| TxError::FetchNotFoundError { message: format!("Cannot find model with id [{id}]") }),
            Err(e) => Err(e)
        }
    }

    fn fetch_option_one(&self, id: &Self::IdType) -> Result<Option<Model<Self::IdType, Data>>, TxError> {
        Ok(self.map.borrow().get(id).map(|val| (*val).clone()))
    }

    fn fetch_version(&self, id: &Self::IdType) -> Result<VersionType, TxError> {
        match self.fetch_option_version(id) {
            Ok(opt) => opt.ok_or_else(|| TxError::FetchNotFoundError { message: format!("Cannot find model with id [{id}]") }),
            Err(e) => Err(e)
        }
    }

    fn fetch_option_version(&self, id: &Self::IdType) -> Result<Option<VersionType>, TxError> {
        Ok(self.map.borrow().get(id).map(|val| val.version))
    }

    fn update(&self, model: Model<Self::IdType, Data>) -> Result<(), TxError> {
        let mut map = self.map.borrow_mut();
        
        match map.entry(model.id.clone()) {
            Entry::Occupied(mut previous) => {
                let previous_version = previous.get().version;
                if previous_version == model.version {
                    let updated_model = model.into_new_version();
                    previous.insert(updated_model);
                    Ok(())
                } else {
                    Err(TxError::UpdateOptimisticLockError { message: format!("Cannot update model with id [{}]. Expected version [{}], version found [{}]", model.id, model.version, previous_version) })
                }
            },
            Entry::Vacant(_) => Err(TxError::UpdateError { message: format!("Cannot update model with id [{}] because it does not exist.", model.id) }),
        }
    }

    fn delete(&self, id: &Self::IdType) -> Result<(), TxError> {
        match self.delete_option(id) {
            Ok(opt) => if opt {
                Ok(())
            } else {
                Err(TxError::DeleteError { message: format!("Cannot delete model with id [{}] because it does not exist.", id) })
            },
            Err(e) => Err(e)
        }
    }

    fn delete_option(&self, id: &Self::IdType) -> Result<bool, TxError> {
        let mut map = self.map.borrow_mut();
        Ok(map.remove(id).is_some())
    }

    fn save(&self, model: NewModel<Self::IdType, Data>) -> Result<(), TxError> {
        let mut map = self.map.borrow_mut();
        match map.entry(model.id.clone()) {
            Entry::Occupied(_) => Err(TxError::UpdateError { message: format!("Cannot save model with id [{}] because the id is already in use.", model.id) }),
            Entry::Vacant(v) => {
                v.insert(Model::from_new(model.into()));
                Ok(())
            },
        }
    }

}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn save_should_save_a_model() {
        // Arrange
        let backend = HashmapBackend::new();
        let model = NewModel {
            id: 1,
            data: 1123,
        };

        // Act
        backend.save(model.clone()).unwrap();
        let fetched_model = backend.fetch_one(&model.id).unwrap();
        let fetched_model_opt = backend.fetch_option_one(&model.id).unwrap();
        
        // Assert
        assert_eq!(model.id, fetched_model.id);
        assert_eq!(model.data, fetched_model.data);
        assert_eq!(0, fetched_model.version);

        assert_eq!(Some(fetched_model), fetched_model_opt);

    }

    #[test]
    fn save_should_fail_if_key_exists() {
        // Arrange
        let backend = HashmapBackend::new();
        let model = NewModel {
            id: 1,
            data: 1123,
        };

        // Act
        let first_save = backend.save(model.clone());
        let second_save = backend.save(model.clone());
        let fetched_model = backend.fetch_one(&model.id).unwrap();
        
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
        let backend = HashmapBackend::<i32, i32>::new();

        // Act
        let fetched_model = backend.fetch_one(&0);
        
        // Assert
        assert!(fetched_model.is_err());

    }

    #[test]
    fn fetch_option_one_should_return_none_if_missing() {
        // Arrange
        let backend = HashmapBackend::<i32, i32>::new();

        // Act
        let fetched_model = backend.fetch_option_one(&0).unwrap();
        
        // Assert
        assert!(fetched_model.is_none());

    }

    #[test]
    fn should_return_the_version() {
        // Arrange
        let backend = HashmapBackend::new();
        let model = NewModel {
            id: 1,
            data: 1123,
        };

        // Act
        backend.save(model.clone()).unwrap();
        let fetched_version = backend.fetch_version(&model.id).unwrap();
        
        // Assert
        assert_eq!(0, fetched_version);

    }

    #[test]
    fn fetch_version_should_fail_if_missing() {
        // Arrange
        let backend = HashmapBackend::<i32, i32>::new();

        // Act
        let fetched_model = backend.fetch_version(&0);
        
        // Assert
        assert!(fetched_model.is_err());

    }

    #[test]
    fn fetch_option_version_should_return_none_if_missing() {
        // Arrange
        let backend = HashmapBackend::<i32, i32>::new();

        // Act
        let fetched_model = backend.fetch_option_version(&0).unwrap();
        
        // Assert
        assert!(fetched_model.is_none());

    }

    #[test]
    fn update_should_update_a_model() {
        // Arrange
        let backend = HashmapBackend::new();
        let model = NewModel {
            id: 1,
            data: 1111,
        };
        backend.save(model.clone()).unwrap();
        let fetched_model_0 = backend.fetch_one(&model.id).unwrap();

        // Act
        let mut updated_model = fetched_model_0.clone();
        updated_model.data = 2222;
        backend.update(updated_model.clone()).unwrap();
        let fetched_model_1 = backend.fetch_one(&model.id).unwrap();
        
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
        let backend = HashmapBackend::new();
        let model = Model {
            id: 1,
            version: 0,
            data: 1111,
        };

        // Act
        let update_result = backend.update(model.clone());
        let fetched_model = backend.fetch_option_one(&model.id).unwrap();
        
        // Assert
        assert!(update_result.is_err());
        assert!(fetched_model.is_none());

    }

    #[test]
    fn update_should_fail_if_version_mismatch() {
        // Arrange
        let backend = HashmapBackend::new();
        let model = NewModel {
            id: 1,
            data: 1111,
        };
        backend.save(model.clone()).unwrap();
        let fetched_model_0 = backend.fetch_one(&model.id).unwrap();

        // Act
        let result_1 = backend.update(fetched_model_0.clone());
        // this should fail because the version does not match
        let result_2 = backend.update(fetched_model_0.clone());
        
        let fetched_model_1 = backend.fetch_one(&model.id).unwrap();
        let result_3 = backend.update(fetched_model_1.clone());
        let fetched_model_2 = backend.fetch_one(&model.id).unwrap();
        
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
        let backend = HashmapBackend::new();
        let model = NewModel {
            id: 1,
            data: 1123,
        };
        backend.save(model.clone()).unwrap();

        // Act
        let fetched_before = backend.fetch_option_one(&model.id).unwrap();
        let delete_result_1 = backend.delete(&model.id);
        let fetched_after = backend.fetch_option_one(&model.id).unwrap();
        let delete_result_2 = backend.delete(&model.id);
        
        // Assert
        assert!(fetched_before.is_some());
        assert!(delete_result_1.is_ok());
        assert!(fetched_after.is_none());
        assert!(delete_result_2.is_err());

    }

    #[test]
    fn delete_option_should_delete_a_model() {
        // Arrange
        let backend = HashmapBackend::new();
        let model = NewModel {
            id: 1,
            data: 1123,
        };
        backend.save(model.clone()).unwrap();

        // Act
        let fetched_before = backend.fetch_option_one(&model.id).unwrap();
        let delete_result_1 = backend.delete_option(&model.id).unwrap();
        let fetched_after = backend.fetch_option_one(&model.id).unwrap();
        let delete_result_2 = backend.delete_option(&model.id).unwrap();
        
        // Assert
        assert!(fetched_before.is_some());
        assert!(delete_result_1);
        assert!(fetched_after.is_none());
        assert!(!delete_result_2);

    }

}