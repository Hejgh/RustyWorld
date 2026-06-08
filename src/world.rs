use bevy::prelude::*;
use fastnoise2::{FastNoiseLite, NoiseLite};
use std::collections::HashMap;

// ============ BLOCK PALETTE ============
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Block {
    #[default] Air,
    Grass,
    Dirt,
    Stone,
    Deepslate,
    CopperOre,
}

impl Block {
    pub fn is_solid(&self) -> bool {
        !matches!(self, Block::Air)
    }
    
    pub fn visible_faces(&self) -> bool {
        self.is_solid() // Only solid blocks need rendering
    }
}

// ============ CHUNK SYSTEM ============
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct ChunkPos(pub i32, pub i32); // X, Z only (Y is full height)

impl ChunkPos {
    pub fn from_world(x: f32, z: f32) -> Self {
        Self((x / 16.0).floor() as i32, (z / 16.0).floor() as i32)
    }
}

pub struct Chunk {
    pub pos: ChunkPos,
    pub blocks: Vec<Block>, // 16*256*16 = 65,536 blocks
    pub is_meshed: bool,
    pub entity: Option<Entity>,
}

impl Chunk {
    pub fn new(pos: ChunkPos) -> Self {
        Self {
            pos,
            blocks: vec![Block::Air; 16 * 256 * 16],
            is_meshed: false,
            entity: None,
        }
    }
    
    fn index(x: usize, y: usize, z: usize) -> usize {
        (y * 16 + z) * 16 + x
    }
    
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Block {
        if y >= 256 { return Block::Air; }
        self.blocks[Self::index(x, y, z)]
    }
    
    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) {
        if y < 256 {
            let idx = Self::index(x, y, z);
            self.blocks[idx] = block;
        }
    }
}

// ============ WORLD RESOURCE ============
pub struct WorldChunks {
    pub chunks: HashMap<ChunkPos, Chunk>,
    pub render_distance: i32,
    pub noise: FastNoiseLite,
    pub biome_noise: FastNoiseLite,
}

impl WorldChunks {
    pub fn new() -> Self {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(fastnoise2::NoiseType::Simplex));
        noise.set_frequency(Some(0.01));
        
        let mut biome_noise = FastNoiseLite::new();
        biome_noise.set_noise_type(Some(fastnoise2::NoiseType::Simplex));
        biome_noise.set_frequency(Some(0.001));
        
        Self {
            chunks: HashMap::new(),
            render_distance: 8,
            noise,
            biome_noise,
        }
    }
    
    pub fn get_or_generate_chunk(&mut self, pos: ChunkPos) -> &mut Chunk {
        if !self.chunks.contains_key(&pos) {
            self.chunks.insert(pos, self.generate_chunk(pos));
        }
        self.chunks.get_mut(&pos).unwrap()
    }
    
    fn generate_chunk(&self, pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new(pos);
        
        for local_x in 0..16 {
            for local_z in 0..16 {
                let world_x = (pos.0 * 16) + local_x as i32;
                let world_z = (pos.1 * 16) + local_z as i32;
                
                // Determine biome at this X,Z
                let biome_val = self.biome_noise.get_noise_2d(world_x as f32, world_z as f32);
                let biome = if biome_val < -0.3 {
                    Biome::Desert
                } else if biome_val < 0.3 {
                    Biome::Plains
                } else {
                    Biome::Swamp
                };
                
                // Heightmap
                let height = self.get_height(world_x, world_z, &biome);
                
                for y in 0..256 {
                    let block = self.get_block_at(height, y, world_x, world_z, &biome);
                    chunk.set_block(local_x as usize, y as usize, local_z as usize, block);
                }
            }
        }
        
        chunk
    }
    
    fn get_height(&self, x: i32, z: i32, biome: &Biome) -> i32 {
        let base = self.noise.get_noise_2d(x as f32, z as f32);
        let elevation = (base * 20.0) as i32 + 64;
        
        match biome {
            Biome::Desert => (base * 15.0) as i32 + 60,
            Biome::Plains => elevation,
            Biome::Swamp => (base * 8.0) as i32 + 58,
        }
    }
    
    fn get_block_at(&self, height: i32, y: i32, x: i32, z: i32, biome: &Biome) -> Block {
        if y > height {
            return Block::Air;
        }
        
        if y == height {
            return match biome {
                Biome::Desert => Block::Sand, // You'd add Sand to Block enum
                Biome::Plains => Block::Grass,
                Biome::Swamp => Block::Grass,
            };
        }
        
        if y > height - 4 {
            return Block::Dirt;
        }
        
        if y < -64 {
            Block::Deepslate
        } else if y < 0 {
            Block::Stone
        } else {
            Block::Stone
        }
    }
}

enum Biome {
    Desert,
    Plains,
    Swamp,
}

// ============ CHUNK MANAGEMENT SYSTEMS ============
pub fn manage_chunks(
    mut world_chunks: ResMut<WorldChunks>,
    player_query: Query<&Transform, (With<Player>, Without<ChunkHandle>)>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_chunk = ChunkPos::from_world(player_transform.translation.x, player_transform.translation.z);
    
    // Load nearby chunks
    for dx in -world_chunks.render_distance..=world_chunks.render_distance {
        for dz in -world_chunks.render_distance..=world_chunks.render_distance {
            let chunk_pos = ChunkPos(player_chunk.0 + dx, player_chunk.1 + dz);
            let chunk = world_chunks.get_or_generate_chunk(chunk_pos);
            
            if chunk.entity.is_none() {
                // Spawn chunk entity with mesh
                let entity = commands.spawn(ChunkHandle { pos: chunk_pos }).id();
                chunk.entity = Some(entity);
            }
        }
    }
    
    // Unload far chunks
    let to_remove: Vec<ChunkPos> = world_chunks.chunks.iter()
        .filter(|(pos, chunk)| {
            chunk.entity.is_some() && 
            (pos.0 - player_chunk.0).abs() > world_chunks.render_distance + 2 ||
            (pos.1 - player_chunk.1).abs() > world_chunks.render_distance + 2
        })
        .map(|(pos, _)| *pos)
        .collect();
    
    for pos in to_remove {
        if let Some(chunk) = world_chunks.chunks.get_mut(&pos) {
            if let Some(entity) = chunk.entity.take() {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component)]
pub struct ChunkHandle {
    pub pos: ChunkPos,
}

#[derive(Component)]
pub struct Player;
