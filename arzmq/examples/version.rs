fn main() {
    let (major, minor, patch) = arzmq::version();
    println!("Current 0MQ version is {major}.{minor}.{patch}");
}
