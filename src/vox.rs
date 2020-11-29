use std::cmp;

use bevy::prelude::Color;
use ndarray::{Array3, Ix};

pub type MaterialId = u8;

pub type Palette = [Material; 256];

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub color: Color,
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
                voxels[position] = voxel.i + 1;
            }
            Model { palette, voxels }
        })
        .collect())
}

fn palette_from_colors(colors: impl IntoIterator<Item=u32>) -> Palette {
    let mut materials = [Material { color: Color::NONE }; 256];
    for (i, color) in colors.into_iter().take(materials.len() - 1).enumerate() {
        materials[i + 1] = Material {
            color: color_from_u32(color),
        };
    }
    materials
}

fn color_from_u32(color: u32) -> Color {
    let component = |i: u8| {
        let offset = i * 8;
        ((color >> offset) & 255) as f32 / 255.0
    };
    Color::rgba(component(0), component(1), component(2), component(3))
}

fn min_max<T>(xs: impl IntoIterator<Item=T>) -> Option<(T, T)>
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
