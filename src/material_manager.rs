use crate::material::Material;

#[derive(Copy, Clone)]
pub struct MaterialHandle(usize);

pub struct MaterialManager {
    materials: Vec<Box<dyn Material + Send + Sync>>
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            materials: Vec::new(),
        }
    }

    pub fn create_material<T: 'static + Material + Send + Sync>(&mut self, object: T)
        -> MaterialHandle {
        
        let new_index = self.materials.len();
        self.materials.push(Box::new(object));

        MaterialHandle(new_index)
    }

    pub fn get_material(&self, material: MaterialHandle) -> &dyn Material {
        self.materials[material.0].as_ref()
    }
}
