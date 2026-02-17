use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn randomize_msgs_order<T>(msgs: &mut [T]) {
    let mut rng = thread_rng();
    msgs.shuffle(&mut rng);
}
