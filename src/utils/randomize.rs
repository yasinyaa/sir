use rand::rng;
use rand::seq::SliceRandom;

pub fn randomize_msgs_order<T>(msgs: &mut [T]) {
    let mut rng = rng();
    msgs.shuffle(&mut rng);
}
