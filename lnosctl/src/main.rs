mod provider;
use provider::*;

fn main() {
    let provider = Provider::new();
    provider.parse_args();
}
