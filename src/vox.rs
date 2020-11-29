use ndarray::{Array3, Ix};

pub type MaterialId = u8;

pub type Palette = [Material; MaterialId::MAX as usize];

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub color: u32,
}

pub struct Model {
    pub palette: Palette,
    pub voxels: Array3<MaterialId>,
}

pub fn load(filename: &str) -> Result<Vec<Model>, &'static str> {
    let data = dot_vox::load(filename)?;

    let mut palette = [Material { color: 0 }; u8::MAX as usize];
    for (index, color) in data.palette.iter().take(palette.len() - 1).enumerate() {
        palette[index + 1] = Material { color: *color };
    }

    Ok(data
        .models
        .iter()
        .map(|model| {
            let min_x = model.voxels.iter().map(|voxel| voxel.x).min().unwrap_or(0);
            let max_x = model.voxels.iter().map(|voxel| voxel.x).max().unwrap_or(0);
            let min_y = model.voxels.iter().map(|voxel| voxel.y).min().unwrap_or(0);
            let max_y = model.voxels.iter().map(|voxel| voxel.y).max().unwrap_or(0);
            let min_z = model.voxels.iter().map(|voxel| voxel.z).min().unwrap_or(0);
            let max_z = model.voxels.iter().map(|voxel| voxel.z).max().unwrap_or(0);
            let mut voxels = Array3::default((
                Ix::from(max_x - min_x + 1),
                Ix::from(max_y - min_y + 1),
                Ix::from(max_z - min_z + 1),
            ));
            for voxel in &model.voxels {
                let position = (
                    Ix::from(voxel.x - min_x),
                    Ix::from(voxel.y - min_y),
                    Ix::from(voxel.z - min_z),
                );
                voxels[position] = voxel.i;
            }
            Model { palette, voxels }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main() {
        for model in load("assets/2x2x2.vox").unwrap() {
            for material_id in model.voxels.iter() {
                println!(
                    "{:?}, {:?}",
                    material_id,
                    model.palette[usize::from(*material_id)]
                );
            }
        }
    }
}
