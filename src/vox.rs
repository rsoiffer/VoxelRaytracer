use ndarray::{Array3, Ix};
use std::cmp;

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
    let palette = palette_from_colors(data.palette);

    Ok(data
        .models
        .iter()
        .map(|model| {
            let (min_x, max_x) =
                min_max(model.voxels.iter().map(|voxel| voxel.x)).unwrap_or((0, 0));
            let (min_y, max_y) =
                min_max(model.voxels.iter().map(|voxel| voxel.y)).unwrap_or((0, 0));
            let (min_z, max_z) =
                min_max(model.voxels.iter().map(|voxel| voxel.z)).unwrap_or((0, 0));
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

fn palette_from_colors(colors: impl IntoIterator<Item = u32>) -> Palette {
    let mut materials = [Material { color: 0 }; u8::MAX as usize];
    for (i, color) in colors.into_iter().take(materials.len() - 1).enumerate() {
        materials[i + 1] = Material { color: color };
    }
    materials
}

fn min_max<T>(xs: impl IntoIterator<Item = T>) -> Option<(T, T)>
where
    T: Copy,
    T: Ord,
{
    xs.into_iter().fold(None, |acc, x| match acc {
        None => Some((x, x)),
        Some((min, max)) => Some((cmp::min(min, x), cmp::max(max, x))),
    })
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
