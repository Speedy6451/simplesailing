use super::MAP_HEIGHT;

use super::MAP_WIDTH;

use libm::Libm;

use nalgebra::Vector2;

pub(crate) fn sample_map_inter(point: Vector2<f32>, map: &[u8]) -> f32 {
    let x = libm::floorf(point.x) as usize % MAP_WIDTH;
    let y = libm::floorf(point.y) as usize % MAP_HEIGHT;
    let p0 = Vector2::new(libm::floorf(point.x), libm::floorf(point.y));
    let p1 = p0 + Vector2::new(1.0, 0.0);
    let p2 = p0 + Vector2::new(0.0, 1.0);
    let p3 = p0 + Vector2::new(1.0, 1.0);

    let tx = point.x - libm::floorf(point.x);
    let ty = point.y - libm::floorf(point.y);

    let top = (1.0 - tx) * sample_map(p0, map) as f32 + tx * sample_map(p1, map) as f32;
    let bot = (1.0 - tx) * sample_map(p2, map) as f32 + tx * sample_map(p3, map) as f32;

    (1.0-ty) * top + ty * bot
}

pub(crate) fn sample_map(point: Vector2<f32>, map: &[u8]) -> u8 {
    const MARGIN: usize = 3;
    let x = libm::floorf(point.x) as usize % (MAP_WIDTH + MARGIN);
    let y = libm::floorf(point.y) as usize % (MAP_HEIGHT + MARGIN);

    if x >= MAP_WIDTH || y >= MAP_HEIGHT { 
        if x-y > 12 {
            return 1;
        }
        return 0; 
    }

    map[y*MAP_WIDTH + x]

}
