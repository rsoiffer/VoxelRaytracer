use ndarray::{Array3, Ix};

#[derive(Clone, Debug, Default)]
struct MaterialId(u8);

impl MaterialId {
    const SIZE: usize = u8::MAX as usize;
}

#[derive(Clone, Copy, Debug)]
struct Material {
    color: u32,
}

struct Model {
    materials: [Material; MaterialId::SIZE],
    voxels: Array3<MaterialId>,
}

fn main() {
    let vox = dot_vox::load("assets/2x2x2.vox").unwrap();

    let mut materials = [Material { color: 0 }; MaterialId::SIZE];
    for (index, color) in vox.palette.iter().take(MaterialId::SIZE - 1).enumerate() {
        materials[index + 1] = Material { color: *color };
    }

    for model in vox.models {
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
        for voxel in model.voxels {
            let position = (
                Ix::from(voxel.x - min_x),
                Ix::from(voxel.y - min_y),
                Ix::from(voxel.z - min_z),
            );
            voxels[position] = MaterialId(voxel.i);
        }

        let model = Model { materials, voxels };
        for material_id in model.voxels.iter() {
            println!(
                "{:?}, {:?}",
                material_id,
                model.materials[usize::from(material_id.0)]
            );
        }
    }
}
