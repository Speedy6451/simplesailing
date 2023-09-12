use libm::Libm;
use nalgebra::Vector2;

pub type PerlinBuf = [u32; 512];

pub fn generate(seed: u32) -> PerlinBuf {
    let mut rand = seed;
    let mut data: PerlinBuf = [0; 512];
    for item in data.iter_mut() {
        rand = xorshift(rand);
        *item = rand;
    }

    data
}

fn xorshift(state: u32) -> u32 {
    // impl from wikipedia
    let mut state = state;
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    state
}

fn fade(t: f32) -> f32 {
    t*t*t*(t*(t*6.0-15.0)+10.0)
}

fn lerp(a: f32, b:f32, d:f32) -> f32 {
    a * (1.0-d) + b * d
}

fn grad(p: Vector2<f32>, b: PerlinBuf) -> Vector2<f32> {
    const width: usize = 16;


    let x = p.x as usize % width;
    let y = p.y as usize % width;

    let one = b[x*width + y] as f32;
    let two = b[(x*width + y + 1)%512] as f32;

    Vector2::new(one, two).normalize()
}

pub fn noise(p: Vector2<f32>, b: PerlinBuf) -> f32 {
    let p0 = Vector2::new(libm::floorf(p.x), libm::floorf(p.y));
    let p1 = p0 + Vector2::new(1.0, 0.0);
    let p2 = p0 + Vector2::new(0.0, 1.0);
    let p3 = p0 + Vector2::new(1.0, 1.0);

    let g0 = grad(p0, b);
    let g1 = grad(p1, b);
    let g2 = grad(p2, b);
    let g3 = grad(p3, b);

    let tx = p.x - p0.x;
    let ftx = fade(tx);
    let ty = p.y - p0.y;
    let fty = fade(ty);

    let p0p1 = (1.0 - ftx) * g0.dot(&(p-p0)) + ftx * g1.dot(&(p-p1));
    let p2p3 = (1.0 - ftx) * g2.dot(&(p-p2)) + ftx * g3.dot(&(p-p3));


    (1.0 - fty) * p0p1 + fty * p2p3
}
