fn main() {
    let (major, minor, patch) = azmq::version();
    println!("Current 0MQ version is {}.{}.{}", major, minor, patch);
}
