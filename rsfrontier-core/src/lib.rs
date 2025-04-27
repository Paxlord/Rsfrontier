pub mod ecd;
pub mod jpk;
pub mod magic;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn say_hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {}
