use candid::{CandidType, Deserialize};

pub type VersionType = u32;

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
pub struct Model<IdType, Data>
{
    pub id: IdType,
    pub(crate) version: VersionType,
    pub data: Data,
}

impl<IdType, Data> Model<IdType, Data> {

    pub(crate) fn from_new(
        model: NewModel<IdType, Data>,
    ) -> Model<IdType, Data> {
        Model {
            id: model.id,
            version: 0,
            data: model.data,
        }
    }

    pub(crate) fn into_new_version(self) -> Model<IdType, Data> {
        Model {
            id: self.id,
            version: self.version + 1,
            data: self.data,
        }
    }

    pub fn version(&self) -> VersionType {
        self.version
    }
}


#[derive(Clone, Debug, PartialEq, CandidType, Deserialize)]
pub struct NewModel<IdType, Data>
{
    pub id: IdType,
    pub data: Data,
}

impl<IdType, Data> NewModel<IdType, Data> {

    pub fn new(
        id: IdType,
        data: Data,
    ) -> Self {
        Self {
            id,
            data,
        }
    }

}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn model_should_impl_debug_if_data_is_debug() {
        let model = Model {
            id: 1,
            version: 1,
            data: SimpleData {
                name: "test".to_owned(),
            },
        };

        println!("Debug model: {:?}", model);
    }

    #[test]
    fn new_model_should_impl_debug_if_data_is_debug() {
        let model = NewModel::new(0, SimpleData {
            name: "test".to_owned(),
        });

        println!("Debug model: {:?}", model);
    }

    #[test]
    fn should_build_new_model_version() {
        let model = Model {
            id: 10,
            version: 10,
            data: SimpleData {
                name: "test".to_owned(),
            },
        };

        let new_model_version = model.clone().into_new_version();

        assert_eq!(model.data, new_model_version.data);
        assert_eq!(model.id, new_model_version.id);
        assert_eq!(model.version + 1, new_model_version.version);
    }

    #[derive(Clone, PartialEq, Debug)]
    struct SimpleData {
        name: String,
    }
}
