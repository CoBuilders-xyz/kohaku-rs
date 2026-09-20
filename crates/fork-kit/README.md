# kohaku-fork-kit

Fixtures for testing Kohaku crates against local network instances.

Kohaku's integration tests are designed to run entirely locally. Certain tests thus require local infrastructure, such as a local Ethereum node, 4337 bundler, or certain deployed contracts. This crate provides APIs for spinning up and interacting with these local instances, as well as fixtures for common test scenarios. 
