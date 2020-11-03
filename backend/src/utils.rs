use rand::Rng;

pub fn get_rand_token<const N: usize>() -> [u8; N] {
    let mut res = [0; N];

    let mut valid_chars = ('a'..='z').chain('A'..='Z').chain('0'..='9').cycle();
    let mut rng = rand::thread_rng();
    for i in 0..N {
        res[i] = valid_chars.nth(rng.gen::<usize>() % 100).unwrap() as u8;
    }
    res
}
